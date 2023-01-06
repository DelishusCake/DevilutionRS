use crate::gfx::{Shader, Pipeline, Topology};

const VERTEX_SHADER_BASIC: &str = include_str!("shaders/basic.vert");
const FRAGMENT_SHADER_COLOR: &str = include_str!("shaders/color.frag");
const FRAGMENT_SHADER_TEXTURED: &str = include_str!("shaders/textured.frag");

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Material {
	Color,
	Textured,
}

#[derive(Debug)]
pub struct MaterialMap {
	textured: Pipeline,
	color_lines: Pipeline,
	color_triangles: Pipeline,
}

impl MaterialMap {
	pub fn new() -> anyhow::Result<Self> {
		let vs_bindings = [("Matrices", 0)];
		let vs_basic = Shader::vertex(VERTEX_SHADER_BASIC, Some(&vs_bindings))?;

        let fs_color = Shader::fragment(FRAGMENT_SHADER_COLOR, None)?;
        let fs_textured = Shader::fragment(FRAGMENT_SHADER_TEXTURED, None)?;

        let shaders_color = [ &vs_basic, &fs_color ];
        let shaders_textured = [ &vs_basic, &fs_textured ];

        let textured = Pipeline::new(Topology::Triangles, &shaders_textured)?;
        let color_lines = Pipeline::new(Topology::Lines, &shaders_color)?;
        let color_triangles = Pipeline::new(Topology::Triangles, &shaders_color)?;

        Ok(Self {
        	textured,
        	color_lines,
        	color_triangles
        })
	}

	pub fn get(&self, topology: Topology, material: Material) -> Option<&Pipeline> {
		match (topology, material) {
			(Topology::Lines,     Material::Color) => Some(&self.color_lines),
			(Topology::Triangles, Material::Color) => Some(&self.color_triangles),
			(Topology::Triangles, Material::Textured) => Some(&self.textured),
			_ => None,
		}
	}
}
