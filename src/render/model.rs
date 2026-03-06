use tobj::{LoadError, Material, Model};

#[derive(Debug, Clone)]
pub enum TobjModelError {
    LoadError(LoadError),
    NoModelFound,
    MoreThanOneModel(usize),
}

pub struct TobjModel {
    model: Model,
    material: Material,
}

impl TobjModel {
    /// Loads a model from an obj file. If the file doesn't contain exactly one model, an error is returned.
    pub fn load_from_obj(path: &str) -> std::result::Result<Self, TobjModelError> {
        let (model, material) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            },
        )
        .map_err(|e| TobjModelError::LoadError(e))?;
        let material = match material {
            Ok(mat) => mat,
            Err(e) => return Err(TobjModelError::LoadError(e)),
        };
        if model.len() == 0 {
            return Err(TobjModelError::NoModelFound);
        } else if model.len() > 1 {
            return Err(TobjModelError::MoreThanOneModel(model.len()));
        }
        Ok(Self {
            model: model.first().unwrap().clone(),
            material: material.first().unwrap().clone(),
        })
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn material(&self) -> &Material {
        &self.material
    }
}
