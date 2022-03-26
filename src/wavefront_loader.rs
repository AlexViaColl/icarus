type Vec2 = (f32, f32);
type Vec3 = (f32, f32, f32);
type Mesh = (Vec<Vertex>, Vec<u32>);
type Vertex = (Vec3, Vec2, Vec3); // pos, uv, normal

pub fn parse_obj(input: &str) -> Result<Mesh, String> {
    let mut vertices = vec![];
    let mut indices = vec![];

    let mut positions = vec![];
    let mut tex_coords = vec![];
    let mut normals = vec![];
    for line in input.lines().map(|l| l.trim()) {
        if line.starts_with("vt") {
            let mut iter = line.split_ascii_whitespace().skip(1);
            let u: f32 = iter.next().unwrap().parse().unwrap();
            let v: f32 = iter.next().unwrap().parse().unwrap();
            tex_coords.push((u, v));
        } else if line.starts_with("vn") {
            let mut iter = line.split_ascii_whitespace().skip(1);
            let x: f32 = iter.next().unwrap().parse().unwrap();
            let y: f32 = iter.next().unwrap().parse().unwrap();
            let z: f32 = iter.next().unwrap().parse().unwrap();
            normals.push((x, y, z));
        } else if line.starts_with('v') {
            let mut iter = line.split_ascii_whitespace().skip(1);
            let x: f32 = iter.next().unwrap().parse().unwrap();
            let y: f32 = iter.next().unwrap().parse().unwrap();
            let z: f32 = iter.next().unwrap().parse().unwrap();
            positions.push((x, y, z));
        } else if line.starts_with('f') {
            let mut iter = line.split_ascii_whitespace().skip(1);
            let mut v0_iter = iter.next().unwrap().split('/');
            let mut v1_iter = iter.next().unwrap().split('/');
            let mut v2_iter = iter.next().unwrap().split('/');

            let v0_pos = v0_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v0_tex = v0_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v0_normal = v0_iter.next().unwrap().parse::<usize>().unwrap() - 1;

            let v1_pos = v1_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v1_tex = v1_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v1_normal = v1_iter.next().unwrap().parse::<usize>().unwrap() - 1;

            let v2_pos = v2_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v2_tex = v2_iter.next().unwrap().parse::<usize>().unwrap() - 1;
            let v2_normal = v2_iter.next().unwrap().parse::<usize>().unwrap() - 1;

            vertices.push((positions[v0_pos], tex_coords[v0_tex], normals[v0_normal]));
            vertices.push((positions[v1_pos], tex_coords[v1_tex], normals[v1_normal]));
            vertices.push((positions[v2_pos], tex_coords[v2_tex], normals[v2_normal]));

            indices.push(indices.len() as u32);
            indices.push(indices.len() as u32);
            indices.push(indices.len() as u32);
        }
    }

    Ok((vertices, indices))
}

pub fn write_obj() -> Result<Vec<u8>, String> {
    Ok(vec![])
}
