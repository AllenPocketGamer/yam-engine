pub trait Asset: Send + Sync + 'static {
    // fn to_raw(&self) -> &[u8];

    fn from_raw() -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_from(path: &str) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn to_raw(&self) -> &[u8] {
        todo!()
    }

    fn save_to(&self, path: &str) {
        todo!()
    }
}

pub struct Texture {
    // TODO!
}

impl Texture {
    // TODO!
}

impl Asset for Texture {}
