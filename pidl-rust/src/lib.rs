use pidl_core::{Arg, Interface, Sig};
use proc_macro2::{Span, TokenStream};
use quasiquote::quasiquote;
use quote::{format_ident, quote};
pub struct Opts {
    pub root: TokenStream,
    pub rpc: Option<RpcOpts>,
}
pub struct RpcOpts {}
pub fn ty(opts: &Opts, i: &Interface, tyx: &Arg, unsync: bool) -> syn::Result<TokenStream> {
    match tyx {
        Arg::Prim { name, ann, params } => match name.as_str() {
            "option" => {
                let p = ty(opts, i, &params[0], unsync)?;
                return Ok(quasiquote!(Option<#p>));
            }
            _ => name.parse().map_err(|e| {
                syn::Error::new(
                    Span::call_site(),
                    format!("invalid primitive (rust parsing: {e})"),
                )
            }),
        },
        Arg::Resource { ty, ann } => {
            let id = match ty {
                pidl_core::ResTy::Of(a) => *a,
                pidl_core::ResTy::This => i.rid(),
            };
            let id = hex::encode(id);
            let name = format_ident!("{}R{id}", if unsync { "Async" } else { "" });
            Ok(quasiquote! {
                impl #name<C>
            })
        }
    }
}
pub fn rpc_ty(opts: &Opts,rpc: &RpcOpts, i: &Interface, tyx: &Arg) -> syn::Result<TokenStream>{
    match tyx{
        Arg::Prim { name, ann, params } => match name.as_str(){
            "option" => {
                let p = rpc_ty(opts,rpc, i, &params[0])?;
                return Ok(quasiquote!(Option<#p>));
            }
            _ => name.parse().map_err(|e| {
                syn::Error::new(
                    Span::call_site(),
                    format!("invalid primitive (rust parsing: {e})"),
                )
            }),
        },
        Arg::Resource { ty, ann } => {
            let id = match ty {
                pidl_core::ResTy::Of(a) => *a,
                pidl_core::ResTy::This => i.rid(),
            };
            let id = hex::encode(id);
            Ok(quasiquote!(#{format_ident!("R{id}RpcId")}))
        },
    }
}
pub fn meth(
    opts: &Opts,
    i: &Interface,
    a: &str,
    b: &Sig,
    unsync: bool,
) -> syn::Result<TokenStream> {
    let ai = format_ident!("{a}");
    let params = b
        .params
        .iter()
        .map(|a| ty(opts, i, a, unsync))
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .enumerate()
        .map(|(a, b)| {
            quasiquote! {
                #{format_ident!("p{a}")}: #b
            }
        });
    let results = b
        .rets
        .iter()
        .map(|a| ty(opts, i, a, unsync))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quasiquote! {
       fn #ai(&mut self,ctx: C::Of<'_>, #(#params),*) -> #{
        let mut v = quote!{
            (#(#results),*)
        };
        if unsync{
            v = quote! {
                impl Future<Output = #v> + Send
            }
        }
        v
       }
    })
}
pub fn rpc_meth(
    opts: &Opts,
    rpc: &RpcOpts,
    i: &Interface,
    a: &str,
    b: &Sig,
    ty: RpcType,
) -> syn::Result<TokenStream> {
    let x = match ty {
        RpcType::Request => &b.params,
        RpcType::Response => &b.rets,
    };
    Ok(quasiquote! {
        #{format_ident!("{a}")}(#{
            let fields = x.iter().map(|a|rpc_ty(opts,rpc,i,a)).collect::<syn::Result<Vec<_>>>()?;
            quote!{#(#fields),*}
        })
    })
}
pub enum RpcType {
    Request,
    Response,
}
pub fn rustify(opts: &Opts, i: &Interface) -> syn::Result<TokenStream> {
    fn inner(opts: &Opts, i: &Interface, unsync: bool) -> syn::Result<TokenStream> {
        let name = format_ident!("{}R{}", if unsync { "Async" } else { "" }, i.rid_str());

        let methods = i.methods.iter().map(|(a, b)| meth(opts, i, a, b, unsync));
        let methods = methods.collect::<syn::Result<Vec<_>>>()?;

        Ok(quasiquote! {
            pub trait #name<C: #{&opts.root}::Budding>: 'static{
                #(#methods)*
            }
        })
    }
    fn rpc(opts: &Opts, i: &Interface) -> syn::Result<TokenStream> {
        let Some(rpc) = opts.rpc.as_ref() else {
            return Ok(quote! {});
        };
        Ok(quasiquote! {
            pub struct #{format_ident!("R{}RpcId",i.rid_str())}{
                pub uuuid: [u8;32]
            }
            pub enum #{format_ident!("R{}RpcReq",i.rid_str())}{
                #{
                    let methods = i.methods.iter().map(|(a,b)|rpc_meth(opts,rpc,i,a,b,RpcType::Request)).collect::<syn::Result<Vec<_>>>()?;
                    quote!{
                        #(#methods),*
                    }
                }
            }
            pub enum #{format_ident!("R{}RpcResp",i.rid_str())}{
                #{
                    let methods = i.methods.iter().map(|(a,b)|rpc_meth(opts,rpc,i,a,b,RpcType::Response)).collect::<syn::Result<Vec<_>>>()?;
                    quote!{
                        #(#methods),*
                    }
                }
            }
        })
    }
    Ok(quasiquote! {
        #{inner(opts,i,true)?}
        #{inner(opts,i,false)?}
        #{rpc(opts,i)?}
    })
}
