use std::path::PathBuf;

use bytemuck::bytes_of;
use glam::Mat4;
pub trait Concatable {
    fn concat(&mut self, text: &str) -> &str;
}

impl Concatable for String {
    fn concat(&mut self, text: &str) -> &str {
        self.push_str(text);
        self.as_str()
    }
}

pub fn get_matrix_as_bytes(mat: &Mat4) -> &[u8] {
    bytes_of(mat)
}

pub fn get_relative_path() -> PathBuf {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use hyakou_core::types::Size;

    #[test]
    fn test_clamped_for_gpu_keeps_dimensions_non_zero() {
        let size = Size {
            width: 0,
            height: 0,
        };

        assert_eq!(
            size.clamp_size_for_gpu(),
            Size {
                width: 1,
                height: 1,
            }
        );
    }
}
