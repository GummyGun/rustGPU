all:
	cargo build

shader:
	glslang -V ssrc/sh.vert -o ssrc/sh.vert.spv
	glslang -V ssrc/sh.frag -o ssrc/sh.frag.spv
