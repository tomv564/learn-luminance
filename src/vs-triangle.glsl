// vertex attributes
in vec2 position;
in vec3 color;

// the only return, color of the pixel
out vec3 v_color;

void main() {
	v_color = color;

	// map xy position to 3d gl point
	gl_Position = vec4(position, 0., 1.);
}