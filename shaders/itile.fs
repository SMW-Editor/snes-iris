#version 400
flat in int g_tile_id;
flat in int g_params;
in vec2 g_tex_coords;

uniform vec2 screen_size;

uniform sampler2D texture;
uniform int tile_size;

out vec4 out_color;

void main() {
    int scale = g_params & 0xFF;
    int tile_id = g_tile_id;

	ivec2 tile_coords = ivec2(tile_id & 0xF, tile_id >> 4) * tile_size;

    ivec2 icoord = ivec2(g_tex_coords);

    icoord = icoord * tile_size / scale;


    out_color = texelFetch(texture, tile_coords + icoord, 0);

	int sel = (g_params >> 16) & 0x3;

	// TODO: debranch
    if (sel == 1) {
    	vec4 color_sel = vec4(0.5,0.5,1.0,1.0);
    	//if (px == 0) { out_color = vec4(0,0,0,0); }
		out_color = mix(out_color, color_sel, 0.4);
		out_color.b += 0.4;
    } else if (sel == 2) {
    	vec4 color_sel = vec4(1.0,0.5,0.5,0.4);
    	//if (px == 0) { out_color = vec4(0,0,0,0); }
		out_color = mix(out_color, color_sel, 0.4);
		out_color.r += 0.4;
    } else if (sel == 3) {
		out_color.a = 0.5;
    } else {
		//if (px == 0) { discard; }
    }

}
