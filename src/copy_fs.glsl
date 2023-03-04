in vec2 v_uv;

out vec4 fragColor;

uniform sampler2D source_texture;

void main() {
	// vec2 texSize = textureSize(source_texture, 0).xy;
	// vec2 texCoord = gl_FragCoord.xy / texSize;


	int blurSize = 1;//int(parameters.x);
	vec2 texSize = textureSize(source_texture, 0).xy;

	//
	if (blurSize <= 0) {
		fragColor = vec4(texture(source_texture, v_uv).rgb, 1.);
		// fragColor = vec4(gammaCorrection(fragColor, 2.2), 1.);
		fragColor = pow(fragColor, vec4(1./2.2));
		return;
	} else {
		for (int i = -blurSize; i <= blurSize; ++i) {
			for (int j = -blurSize; j <= blurSize; ++j) {
				vec2 offset = vec2(i, j) / texSize;
				fragColor += vec4(texture(source_texture, v_uv + offset).rgb, 1.);
				// fragColor = pow(fragColor, vec4(1./2.2));
			}
		}

		// if blurSize = 3, then we've sampled 7^2 = 49 pixels?
		fragColor /= pow(blurSize * 2 + 1, 2);
		// vec2 offset = vec2(2., 2.) / texSize;
		// fragColor = vec4(texture(source_texture, v_uv + offset).rgb, 1.);
		// fragColor = pow(fragColor, vec4(1./2.2));
	}



	// frag = vec4(texture(source_texture, v_uv).rgb, 1.);
	// frag = pow(frag, vec4(1./2.2));
}