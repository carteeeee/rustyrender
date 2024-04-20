__kernel void render (
    __global float3 const* const tris,
    __global uchar* const out,
    __private float const fov,
    __private uint const width,
    __private uint const height,
    __private ulong const numtris
) {
    uint const idx = get_global_id(0);
    float const x = idx % width;
    float const y = idx / width;
    float const trig = (height/(2.*tan(radians(fov)/2.))); // wtf is happening here
    float3 const rdir = (float3)(
        (x + 0.5 - (float)width/2.)/trig,
        (-y + 0.5 + (float)height/2.)/trig,
        -1.
    )*0.01f;

    bool stop = false;
    uint end = 255;
    for (uint i = 0; (i < 1000) && !stop; i++) {
        float3 const raypos = rdir * (float)i;
        for (uint t = 0; (t < numtris) && !stop; t++) {
            float3 const tv1 = tris[t * 3];
            float3 const tv2 = tris[t * 3 + 1];
            float3 const tv3 = tris[t * 3 + 2];

            float3 u1 = tv1 - raypos;
            float3 v1 = tv2 - raypos;
            float3 n1 = cross(u1, v1);

            float3 u2 = tv2 - raypos;
            float3 v2 = tv3 - raypos;
            float3 n2 = cross(u2, v2);

            float3 u3 = tv3 - raypos;
            float3 v3 = tv1 - raypos;
            float3 n3 = cross(u3, v3);

            float d1 = dot(n1, n2);
            float d2 = dot(n1, n3);

            if (!(d1 < 0.f) && !(d2 < 0.f)) {
                end = i;
                stop = true;
                break;
            }
        }
    }
    out[idx] = 255-(end/1000.f)*255;
}
