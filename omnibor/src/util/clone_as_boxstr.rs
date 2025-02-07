use {
    std::path::{Path, PathBuf},
    url::Url,
};

pub trait CloneAsBoxstr {
    fn clone_as_boxstr(self) -> Box<str>;
}

impl CloneAsBoxstr for &str {
    fn clone_as_boxstr(self) -> Box<str> {
        self.to_owned().into_boxed_str()
    }
}

impl CloneAsBoxstr for &String {
    fn clone_as_boxstr(self) -> Box<str> {
        self.clone().into_boxed_str()
    }
}

impl CloneAsBoxstr for &Path {
    fn clone_as_boxstr(self) -> Box<str> {
        self.display().to_string().into_boxed_str()
    }
}

impl CloneAsBoxstr for &PathBuf {
    fn clone_as_boxstr(self) -> Box<str> {
        self.display().to_string().into_boxed_str()
    }
}

impl CloneAsBoxstr for &Url {
    fn clone_as_boxstr(self) -> Box<str> {
        self.clone().to_string().into_boxed_str()
    }
}
