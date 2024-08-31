use higher_kinded_types::ForLifetime;
use higher_kinded_types::ForLt;

pub mod rexport {
    pub extern crate higher_kinded_types;
}
pub trait Bud<'a, C: ForLifetime + ?Sized> {
    fn bud<'b>(&'b mut self) -> C::Of<'b>;
}
impl<'a, C: ?Sized + for<'b> ForLifetime<Of<'b> = &'b mut T>, T: ?Sized> Bud<'a, C> for &'a mut T {
    fn bud<'b>(&'b mut self) -> <C as ForLifetime>::Of<'b> {
        &mut **self
    }
}
impl<'a, C: ?Sized + for<'b> ForLifetime<Of<'b> = ()>> Bud<'a, C> for () {
    fn bud<'b>(&'b mut self) -> <C as ForLifetime>::Of<'b> {
        ()
    }
}

#[cfg(feature = "wasm_runtime_layer")]
const _: () = {
    use wasm_runtime_layer::{backend::WasmEngine, AsContextMut, StoreContextMut};
    impl<
            'a,
            E: WasmEngine,
            U,
            C: ?Sized + for<'b> ForLifetime<Of<'b> = StoreContextMut<'b, U, E>>,
        > Bud<'a, C> for StoreContextMut<'a, U, E>
    {
        fn bud<'b>(&'b mut self) -> <C as ForLifetime>::Of<'b> {
            self.as_context_mut()
        }
    }
};
pub trait Budding: for<'a> ForLifetime<Of<'a>: Bud<'a, Self>> {}
impl<T: for<'a> ForLifetime<Of<'a>: Bud<'a, Self>>> Budding for T {}
