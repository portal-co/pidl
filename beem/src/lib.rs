#![no_std]
use embedded_io::{ErrorType, Read, ReadExactError, Write};

use weev::Stream;
use sha3::Digest;

pub trait Beem: ErrorType + Sized{
    fn accept(&mut self, seed: &[u8]) -> Result<Self,Self::Error>;
    fn offer(&mut self) -> Result<Self,ReadExactError<Self::Error>>;
}
pub trait AsyncBeem: ErrorType + Sized{
    async fn accept(&mut self, seed: &[u8]) -> Result<Self,Self::Error>;
    async fn offer(&mut self) -> Result<Self,ReadExactError<Self::Error>>;
}
impl<S: Clone,Q: Clone> Beem for Stream<S,Q> where Stream<S,Q>: embedded_io::Read + embedded_io::Write{
    fn accept(&mut self, seed: &[u8]) -> Result<Self,Self::Error> {
        let seed: [u8;32] = sha3::Sha3_256::digest(seed).try_into().unwrap();
        self.write_all(&seed)?;
        let mut n = sha3::Sha3_256::new();
        n.update(seed);
        n.update(self.sid);
        let n = n.finalize().try_into().unwrap();
        Ok(Self { core: self.core.clone(), sid: n })
    }

    fn offer(&mut self) -> Result<Self, ReadExactError<Self::Error>> {
        let mut seed = [0u8;32];
        self.read_exact(&mut seed)?;
        let mut n = sha3::Sha3_256::new();
        n.update(seed);
        n.update(self.sid);
        let n = n.finalize().try_into().unwrap();
        Ok(Self { core: self.core.clone(), sid: n })
    }
}
impl<S: Clone,Q: Clone> AsyncBeem for Stream<S,Q> where Stream<S,Q>: embedded_io_async::Read + embedded_io_async::Write{
    async fn accept(&mut self, seed: &[u8]) -> Result<Self,Self::Error> {
        use embedded_io_async::Write;
        let seed: [u8;32] = sha3::Sha3_256::digest(seed).try_into().unwrap();
        self.write_all(&seed).await?;
        let mut n = sha3::Sha3_256::new();
        n.update(seed);
        n.update(self.sid);
        let n = n.finalize().try_into().unwrap();
        Ok(Self { core: self.core.clone(), sid: n })
    }

    async fn offer(&mut self) -> Result<Self, ReadExactError<Self::Error>> {
        use embedded_io_async::Read;
        let mut seed = [0u8;32];
        self.read_exact(&mut seed).await?;
        let mut n = sha3::Sha3_256::new();
        n.update(seed);
        n.update(self.sid);
        let n = n.finalize().try_into().unwrap();
        Ok(Self { core: self.core.clone(), sid: n })
    }
}