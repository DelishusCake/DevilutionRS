use crate::{Pipeline, Shader, Topology};

/// Embed the shader source directly in the binary
/// This is arguably rust's best feature, it alone makes it worth it to use rust instead of C
const VERTEX_SHADER_BASIC: &str = include_str!("shaders/basic.vert");
const FRAGMENT_SHADER_COLOR: &str = include_str!("shaders/color.frag");
const FRAGMENT_SHADER_TEXTURED: &str = include_str!("shaders/textured.frag");
const FRAGMENT_SHADER_TEXTURED_ARRAY: &str = include_str!("shaders/textured_array.frag");

/// Material type enums
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Material {
    /// Colored, but not textured geometry
    Color,
    /// Textured geometry
    Textured,
    /// Textured geometry using a texture array
    LayeredTexture,
}

/// Material map
/// Used to look up the singleton pipeline objects for a given material
#[derive(Debug)]
pub struct MaterialMap {
    textured: Pipeline,
    textured_array: Pipeline,
    color_lines: Pipeline,
    color_triangles: Pipeline,
}

impl MaterialMap {
    /// Create a new material map object
    pub fn new() -> anyhow::Result<Self> {
        // Vertex shader creation
        // Vertex shader bindings, in (name, location) pair form
        let vs_bindings = [("Matrices", 0)];
        // The basic vertex shader
        let vs_basic = Shader::vertex(VERTEX_SHADER_BASIC, Some(&vs_bindings))?;
        // Fragment shader creation
        // Basic color-only fragment shader
        let fs_color = Shader::fragment(FRAGMENT_SHADER_COLOR, None)?;
        // Fragment shader for textured geometry
        let fs_textured = Shader::fragment(FRAGMENT_SHADER_TEXTURED, None)?;
        let fs_textured_array = Shader::fragment(FRAGMENT_SHADER_TEXTURED_ARRAY, None)?;

        // Shader list describing the colored geometry pipeline
        let shaders_color = [&vs_basic, &fs_color];
        // Shader list describing the textured geometry pipeline
        let shaders_textured = [&vs_basic, &fs_textured];
        let shaders_textured_array = [&vs_basic, &fs_textured_array];

        // Textured triangles pipeline
        // NOTE: It doesn't make much sense to have a line topoly version of this
        let textured = Pipeline::new(Topology::Triangles, &shaders_textured)?;
        let textured_array = Pipeline::new(Topology::Triangles, &shaders_textured_array)?;
        // Colored lines pipeline
        let color_lines = Pipeline::new(Topology::Lines, &shaders_color)?;
        // Colored triangles pipeline
        let color_triangles = Pipeline::new(Topology::Triangles, &shaders_color)?;

        Ok(Self {
            textured,
            textured_array,
            color_lines,
            color_triangles,
        })
    }

    // Get the pipeline for a meterial and topology
    pub fn get(&self, topology: Topology, material: Material) -> Option<&Pipeline> {
        match (topology, material) {
            (Topology::Lines, Material::Color) => Some(&self.color_lines),
            (Topology::Triangles, Material::Color) => Some(&self.color_triangles),
            (Topology::Triangles, Material::Textured) => Some(&self.textured),
            (Topology::Triangles, Material::LayeredTexture) => Some(&self.textured_array),
            _ => None,
        }
    }
}
