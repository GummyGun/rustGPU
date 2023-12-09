all:
	cargo build

shader:
	glslang -V s_src/sh.vert -o runtime/sh.vert.spv
