SRC = \
	blur.vert \
	blur.frag \
	compositor.vert \
	compositor.frag \
	particle.vert \
	particle.frag

SPV = $(SRC:%=%.spv)

all: $(SPV)

%.vert.spv: %.vert
	glslc -c $< -o $@

%.frag.spv: %.frag
	glslc -c $< -o $@

clean:
	rm -f $(SPV)

re: clean all
