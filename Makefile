all:
	cargo build

shader:
	glslang -V src/ssrc/sh.vert -o res/shaders/sh.vert.spv
	glslang -V src/ssrc/sh.frag -o res/shaders/sh.frag.spv
	glslang -V src/ssrc/sh.comp -o res/shaders/sh.comp.spv
