all:
	cargo build

shader:
	glslang -V ssrc/sh.vert -o ssrc/sh.vert.spv
