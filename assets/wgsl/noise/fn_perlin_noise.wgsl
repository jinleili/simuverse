// cpu version: https://mrl.cs.nyu.edu/~perlin/noise/
// Implementing Improved Perlin Noise:
// https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-26-implementing-improved-perlin-noise

fn fade(t: vec3<f32>) -> vec3<f32> {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn perm(x: i32, y: i32) -> vec4<i32> {
    return permutation[y * 256 + x];
}

fn grad(x: i32, p: vec3<f32>) -> f32 {
    // only use 16-pixels slightly improve performence.
    return dot(gradient[x & 15].xyz, p);
}

fn lerp(a: f32, b: f32, w: f32) -> f32 {
    return a + (b - a) * w;
}

fn lerp3(a: vec3<f32>, b: vec3<f32>, w: f32) -> vec3<f32> {
    return a + (b - a) * w;
}

fn noise(pos: vec3<f32>) -> f32 {
    // find unit cube
    let P: vec3<i32> = vec3<i32>(floor(pos)) % vec3<i32>(256);  
    // point in unit cube
    let fract_pos = fract(pos);  
    // fade curves  
    let f: vec3<f32> = fade(fract_pos);      
    // HASH COORDINATES FOR 6 OF THE 8 CUBE CORNERS  
    let hash = (perm(P.x, P.y) + P.z) % vec4<i32>(256);

    // BLENDED RESULTS FROM 8 CORNERS OF CUBE  
    return lerp(lerp(lerp(
            grad(hash.x, fract_pos), 
            grad(hash.z, fract_pos + vec3<f32>(-1.0, 0.0, 0.0)), f.x),           
        lerp(
            grad(hash.y, fract_pos + vec3<f32>(0.0, -1.0, 0.0)), 
            grad(hash.w, fract_pos + vec3<f32>(-1.0, -1.0, 0.0)), f.x), f.y),      
        lerp(lerp(
            grad(hash.x + 1, fract_pos + vec3<f32>(0.0, 0.0, -1.0)), 
            grad(hash.z + 1, fract_pos + vec3<f32>(-1.0, 0.0, -1.0)), f.x),           
        lerp(
            grad(hash.y + 1, fract_pos + vec3<f32>(0.0, -1.0, -1.0)), 
            grad(hash.w + 1, fract_pos + vec3<f32>(-1.0, -1.0, -1.0)), f.x), f.y), f.z); 
}
