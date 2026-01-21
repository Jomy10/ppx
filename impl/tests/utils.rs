#[cfg(feature = "vfs")]
mod feature_vfs {
    pub struct VfsWrapper {
        pub path: vfs::VfsPath,
    }

    impl VfsWrapper {
        pub fn join(&self, other: impl AsRef<str>) -> VfsWrapper {
            self.path.join(other.as_ref()).unwrap().into()
        }
    }

    impl From<vfs::VfsPath> for VfsWrapper {
        fn from(value: vfs::VfsPath) -> Self {
            Self { path: value }
        }
    }

    impl Into<vfs::VfsPath> for VfsWrapper {
        fn into(self) -> vfs::VfsPath {
            self.path
        }
    }

    impl AsRef<vfs::VfsPath> for VfsWrapper {
        fn as_ref(&self) -> &vfs::VfsPath {
            &self.path
        }
    }

    pub type OwnedPath = VfsWrapper;
    pub type Path = VfsWrapper;
}

#[cfg(not(feature = "vfs"))]
pub type OwnedPath = std::path::PathBuf;

#[allow(unused)]
#[cfg(not(feature = "vfs"))]
pub type Path = std::path::Path;

#[cfg(feature = "vfs")]
pub type OwnedPath = feature_vfs::OwnedPath;

#[allow(unused)]
#[cfg(feature = "vfs")]
pub type Path = feature_vfs::Path;

pub fn pwd() -> OwnedPath {
    #[cfg(not(feature = "vfs"))]
    return std::env::current_dir().unwrap();
    #[cfg(feature = "vfs")]
    return vfs::VfsPath::new(vfs::PhysicalFS::new(".")).into();
}
