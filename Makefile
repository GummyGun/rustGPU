all:
	cargo build

shader:
	glslang -V src/ssrc/sh.vert -o res/shaders/sh.vert.spv
	glslang -V src/ssrc/sh.frag -o res/shaders/sh.frag.spv
	glslang -V src/ssrc/sh.comp -o res/shaders/sh.comp.spv
	glslang -V src/ssrc/gradient_color.comp -o res/shaders/gradient_color.comp.spv
	glslang -V src/ssrc/sky.comp -o res/shaders/sky.comp.spv
	glslang -V src/ssrc/triangle.vert -o res/shaders/triangle.vert.spv
	glslang -V src/ssrc/triangle.frag -o res/shaders/triangle.frag.spv
	glslang -V src/ssrc/mesh.vert -o res/shaders/mesh.vert.spv
	glslang -V src/ssrc/mesh.frag -o res/shaders/mesh.frag.spv
