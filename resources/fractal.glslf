#version 420 core

#define ESCAPE_BOUNDARY 18
#define FREQUENCY 0.1

in vec4 gl_FragCoord;
out vec4 Target0;


dvec2 viewport_coord_to_complex(dvec2 coord); // moves and scales gl_FragCoord to desired complex number

// For this iterative escape-fractal, we need to select start value, and iterative modifier
dvec2 get_c();
dvec2 get_z();

vec3 hsv2rgb( vec3 c);

layout (std140) uniform MandelShaderUniforms {
  uniform vec2 u_Center;     // where on the complex plain the user wants the screen-center
  uniform vec2 u_Dimension;  // the width and height user wants the view onto the complex plane
  uniform vec2 u_Resolution; // user interogates the pixel coverage of the view, and provides it here

  uniform vec2 u_MousePos;    // Used for julia set, assumes gl_FragCoord format
  uniform float u_Time;       // Time since start of program in seconds
  uniform int u_MaxIteration; // User defined limit of iteration count
  uniform int u_IsMandel;     // (A bit hacky, TODO) select between MandelBrot and Julia
};



void main() {

  // load up our iteration values
  dvec2 z = get_z();
  dvec2 c = get_c();


  int step_count;
  dvec2 tmp;
  for (step_count = 0; step_count < u_MaxIteration; step_count++) {

    // Vast majority of performance gets absorbed here

    // TODO micro-optimise
    // `z = z * z + c`, with z and c being complex numbers

    tmp.x = z.x*z.x - z.y*z.y;
    tmp.y = 2.0 * z.x*z.y;
    z.x = tmp.x;
    z.y = tmp.y;
    z.x += c.x;
    z.y += c.y;

    if((z.x*z.x + z.y*z.y) > ESCAPE_BOUNDARY) {
      break;
    }
  }


  if (step_count == u_MaxIteration) {
    Target0 =  vec4(1.0, 1.0, 1.0, 1.0);
  } else {
    float dist = float(length(z));

    float two = float(2.0);
    dist  = log(log(dist)) / log(two);

    float val = float(step_count) - dist;
    float hue = val/10.0 - u_Time * FREQUENCY;

    Target0 =  vec4(hsv2rgb(vec3((hue), 1.0, 1.0 )), 1.0);
  }
}
// enhance...
dvec2 vec2_to_dvec2(vec2 floatvec2) {
  return dvec2(floatvec2);
}

// center yourself, THEN grow your mind, then you will be moved on the plane of imagination
dvec2 viewport_coord_to_complex(dvec2 coord) {

  // set the origin to the center of the screen
  dvec2 result =  dvec2(coord) - u_Resolution/2.0;


  // Scale down to a 1.0 by 1.0 view of the complex plane
  result /= u_Resolution;

  // match the real and im dimensions to user-input
  result *= u_Dimension;

  // shift the center reference to where the user asked it to be
  result += u_Center;
  return result;
}


// TODO set it up so that responsibility between mandel and julia decoupled
//      at the moment seperate areoas are tightly coupled
dvec2 get_c() {
  dvec2 result;

  // for mandel, the iter-step is where that pixel sits in the complex plane
  // but for julia, it's based on mouse position
  if(u_IsMandel != 0) {
    result = viewport_coord_to_complex(gl_FragCoord.xy);
  } else {
    result = viewport_coord_to_complex(u_MousePos);
  }

  return result;
}

// TODO see get_c()
dvec2 get_z() {
  dvec2 result;

  // for mandel, iterate starting from 0, for julia, the start relates to location in
  // the complex plane
  if(u_IsMandel != 0) {
    result = dvec2(0.0, 0.0);
  } else {
    result = viewport_coord_to_complex(gl_FragCoord.xy);
  }

  return result;
}


// hue, saturation, value to reg, green, blue
vec3 hsv2rgb(vec3 c) {
  vec4 K =  vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www );
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
