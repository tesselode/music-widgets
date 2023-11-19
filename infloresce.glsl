#version 330 core
out vec4 FragColor;

uniform sampler2D ourTexture;
uniform vec4 blendColor;
uniform float iTime;
uniform vec2 iResolution;

vec3 C;
vec2 uv;
vec2 cell;
float TAU=radians(360.);
vec3 p1 = vec3(.3, .7, 1.);
vec3 p2 = vec3(1., .6, .7);

float G = 0.;

#define nsin(X) (.5+.5*sin(X))

float T;
//#define T iTime
#define SLOW nsin(T*.1)
#define MEDIUM nsin(T*.5+3.)
#define FAST nsin(T*.6+4.)

float pmix(float a, float b, float q) {
  return pow(b, q) * pow(a, 1.-q);
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5*(a-b)/k, 0.0, 1.0);
    return mix(a, b, h) - k*h*(1.0-h);
}

float smax(float a, float b, float k) {
  return -smin(-a, -b, k);
}

float sinterp(float x) {
  return 3.*x*x - 2.*x*x*x;
}

mat2 rot(float a) {
  return mat2(cos(a), sin(a), -sin(a), cos(a));
}

float box(vec3 p, vec3 a) {
  vec3 q = abs(p)-a;
  return length(max(q,0.))+min(0., max(q.x,max(q.y,q.z)));
}

float W(vec3 op) {
  G += 1.;
  vec3 p = op.xyz;
  p.xy *= rot(T);
  p.xz *= rot(MEDIUM);
  //p.xy *= rot(length(sin(fract(100.*cell+1.8398624))));
  float b = box(p, vec3(.5))-.2;
  
  op.xz *= rot(T*.1);
  //op.xz *= rot(sin(T*3.));
    
  b = smin(b, length(op.xz)-.2, .7);
  b = smin(b, length(op.yz)-.2, .7);  

  b = smax(b, -(length(p)-.6), .5);

  float outer = length(op+.2) - 2.5;
  
  b = smax(b, outer, .5);
  
  return b;
}

float D;
vec3 P;
void tr(vec3 start, vec3 look) {
  D = 0.;
  float s;
  P = start;
  
  for (float i=0.; i<100.; i+=1.) {
    s = W(P);
    D += s;
    P += look * s;
    if (s < .001) return;
  }
  
  D= 1000.;
}

float grid(vec2 uv) {
  //return 5.;
  uv = sin(uv*20.);
  return pmix(8., 400., nsin(T))*(abs(uv.x)+abs(uv.y));
  //return length(sin((uv+T*.03)*(20.+20.*SLOW)));
}

vec3 norm(vec3 p) {
  mat3 k = mat3(p,p,p) - mat3(.01);
  return normalize(vec3(W(k[0]), W(k[1]), W(k[2])));
}

vec3 render3d(vec3 look) {
    G=0.;
    vec3 cam = vec3(0,0,-5);
    vec3 R = vec3(0);
    
    tr(cam, normalize(look));
    vec3 N = norm(P);
    G /= 3.;
    N.xy *= rot(iTime);
    float diff = length(.5+.5*sin(1.5*N))/sqrt(3.);
    float pl = dot(N, normalize(P));

    if (D < 10.) {
        R += p1*diff*.05;//-.04*f;
        R += p1*pow(pl, 3.)*.1+.05;
        //R.rgb += max(0.,.2*pow(dot(N,normalize(P)),3.));
    }

    return R;
}

void main()
{
    T=iTime;
    T *= .7;
    
    // Normalized pixel coordinates (from 0 to 1)
    vec2 ncuv = gl_FragCoord.xy/iResolution.xy;
    vec2 uv = ncuv - .5;
    
    uv.x *= iResolution.x/iResolution.y;
    float f = .4+2.*nsin(.1*iTime);
    uv *= f;
    ncuv *= f;
    uv *= rot(SLOW);
    uv *= rot(MEDIUM - 1.+ mix(-1., 1., FAST)*length(uv));
    uv *= rot(MEDIUM + 2.+ mix(-.2, .2, MEDIUM)*pow(length(uv),2.));
    uv *= rot(MEDIUM - 10.+mix(-.2, .2, SLOW)*pow(length(uv), 3.));
    uv -= TAU/29.;
    uv *= rot(-SLOW);
    vec2 local = fract(uv*TAU)-.5;
    cell = floor(uv*TAU);
    
    T += uv.y*uv.y;
    float bright = smoothstep(.5+.5*grid(uv), 0., .3 + .3*SLOW);

    vec2 hiuv = rot(sin(iTime)*.1)*uv;
    float hilite = 1.-smoothstep(abs(hiuv.y/f), 0., .05*(MEDIUM+.1));

    C.rgb = (1.1+hilite*(1.-length(uv)))*mix(p1, p2, abs(uv.y/f)+.1);//nsin(uv.xyx);
    
    vec3 rendered = vec3(0);
    if (mod(cell.x + 3.*cell.y, 5.) == 4.) {
        rendered = render3d(vec3(-local,1.));
    }

    C.rgb += rendered;
    
    rendered = render3d(vec3(rot(FAST)*(-uv*.18)-(.1),1.));
    
    C.rgb += nsin(T*.05)*(rendered - G*.01 + .3);
    
    C *= pow(bright, nsin(-T*.05));//bright*bright*bright;
    
    FragColor = vec4(C,1.0) * blendColor;
}
