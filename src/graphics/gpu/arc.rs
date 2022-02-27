use std::{
    any::TypeId,
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc, RwLock,
    },
};

lazy_static::lazy_static! {
    static ref NEXT_ID: RwLock<HashMap<TypeId, AtomicU64>> = RwLock::new(HashMap::new());
}

#[derive(Debug)]
pub struct ArcHandle<T: 'static> {
    pub handle: Arc<T>,
    id: u64,
}

impl<T: 'static> ArcHandle<T> {
    #[allow(unused_results)]
    pub fn new(handle: T) -> Self {
        let lock = NEXT_ID.read().expect("id read lock");
        let id = if let Some(id) = lock.get(&TypeId::of::<T>()) {
            id.fetch_add(1, SeqCst)
        } else {
            std::mem::drop(lock);
            let mut write = NEXT_ID.write().expect("id write lock");
            write.insert(TypeId::of::<T>(), AtomicU64::new(0));
            write.get(&TypeId::of::<T>()).unwrap().fetch_add(1, SeqCst)
        };

        ArcHandle {
            handle: Arc::new(handle),
            id,
        }
    }

    #[inline(always)]
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<T: 'static> Clone for ArcHandle<T> {
    fn clone(&self) -> Self {
        ArcHandle {
            handle: Arc::clone(&self.handle),
            id: self.id,
        }
    }
}

impl<T: 'static> PartialEq for ArcHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: 'static> Eq for ArcHandle<T> {}

impl<T: 'static> std::hash::Hash for ArcHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: 'static> std::ops::Deref for ArcHandle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.handle.as_ref()
    }
}

impl<T: 'static> AsRef<T> for ArcHandle<T> {
    fn as_ref(&self) -> &T {
        self.handle.as_ref()
    }
}

pub type ArcBuffer = ArcHandle<wgpu::Buffer>;
pub type ArcTexture = ArcHandle<wgpu::Texture>;
pub type ArcTextureView = ArcHandle<wgpu::TextureView>;
pub type ArcBindGroupLayout = ArcHandle<wgpu::BindGroupLayout>;
pub type ArcBindGroup = ArcHandle<wgpu::BindGroup>;
pub type ArcPipelineLayout = ArcHandle<wgpu::PipelineLayout>;
pub type ArcRenderPipeline = ArcHandle<wgpu::RenderPipeline>;
pub type ArcSampler = ArcHandle<wgpu::Sampler>;
pub type ArcShaderModule = ArcHandle<wgpu::ShaderModule>;
