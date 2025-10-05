use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::time::Instant;
// use std::collections::HashSet;
// use std::io::Write;
use std::io::BufRead;
// use std::io::BufReader;
// use std::error::Error;
// use std::collections::HashMap;
// use once_cell::sync::Lazy;
// use std::sync::Mutex;
// use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

#[derive(Clone, Debug)]
pub struct Graph {
    pub num_vertices: u8,
    pub edges: Vec<(u8, u8)>,
    pub adj: Vec<u64>, // adjacency matrix as bit-packed rows
}


// utilities
fn inverse(perm: &Vec<u8>) -> Vec<u8> {
    let mut inv = vec![0u8; perm.len()];
    for (i, &p) in perm.iter().enumerate() {
        inv[p as usize] = i as u8;
    }
    inv
}

fn compose(a: &Vec<u8>, b: &Vec<u8>) -> Vec<u8> {
    a.iter().map(|&x| b[x as usize]).collect()
}


impl Graph {
    pub fn new(num_vertices: u8, edges: Vec<(u8, u8)>) -> Self {
        let mut adj = vec![0u64; num_vertices as usize];
        for &(u, v) in &edges {
            adj[u as usize] |= 1u64 << v;
            adj[v as usize] |= 1u64 << u;
        }
        Graph {
            num_vertices,
            edges,
            adj,
        }
    }
    // pub fn _adjacency_matrix(&self) -> Vec<Vec<bool>> {
    //     let n = self.num_vertices as usize;
    //     let mut mat = vec![vec![false; n]; n];
    //     for &(u, v) in &self.edges {
    //         mat[u as usize][v as usize] = true;
    //         mat[v as usize][u as usize] = true;
    //     }
    //     mat
    // }
    // pub fn update_adj(&mut self) {
    //     // if self.adj.is_some() { return; }
    //     let n = self.num_vertices as usize;
    //     let mut m = vec![vec![false; n]; n];
    //     for &(u, v) in &self.edges {
    //         m[u as usize][v as usize] = true;
    //         m[v as usize][u as usize] = true;
    //     }
    //     self.adj = m;
    // }

    #[inline(always)]
    fn adj(&self, u: usize, v: usize) -> bool {
        (self.adj[u] & (1u64 << v)) != 0
    }

    pub fn degrees(&self) -> Vec<usize> {
        (0..self.num_vertices as usize).map(|v| self.adj[v].count_ones() as usize).collect()
        // let mut deg = vec![0; self.num_vertices as usize];
        // for &(u, v) in &self.edges {
        //     deg[u as usize] += 1;
        //     deg[v as usize] += 1;
        // }
        // deg
    }

    pub fn induced_edge_count_mask(&self, mask: u64) -> usize {
        // sum popcount(adj[v] & mask) for v in mask, then /2
        let mut sum = 0usize;
        let mut mm = mask;
        while mm != 0 {
            let lsb = mm.trailing_zeros() as usize;
            sum += (self.adj[lsb] & mask).count_ones() as usize;
            mm &= mm - 1;
        }
        sum / 2
    }

    pub fn degrees2_new(&self, degree_factor: usize) -> Vec<usize> {
        let n = self.num_vertices as usize;
        let mut deg2 = vec![0usize; n];
        for v in 0..n {
            // build neighbor mask including self
            let mask = self.adj[v] | (1u64 << v);
            // induced edges inside mask
            let edges = self.induced_edge_count_mask(mask);
            let neighbors_count = mask.count_ones() as usize;
            deg2[v] = edges + degree_factor * (neighbors_count.saturating_sub(1));
        }
        deg2
    }

    pub fn degrees3_new(&self, degree_factor1: usize, degree_factor2: usize) -> Vec<usize> {
        let n = self.num_vertices as usize;
        let mut deg3 = vec![0usize; n];
        for v in 0..n {
            // compute 2-neighborhood mask
            let mut mask = self.adj[v] | (1u64 << v);
            // add neighbors of neighbors
            let mut mm = self.adj[v];
            while mm != 0 {
                let u = mm.trailing_zeros() as usize;
                mask |= self.adj[u];
                mm &= mm - 1;
            }
            // count edges induced
            let edges = self.induced_edge_count_mask(mask);
            deg3[v] = edges;
        }
        let deg2 = self.degrees2(degree_factor1);
        for v in 0..n {
            deg3[v] += degree_factor2 * deg2[v];
        }
        deg3
    }

    pub fn degrees2(&self, degree_factor: usize) -> Vec<usize> {
        // produces a list, for each vertex of the number of edges in the 1-neighborhood, i.e., 
        // the number of edges adjacent to the vertices and their direct neighbors
        let n = self.num_vertices as usize;
        let mut deg2 = vec![0; n];
        // let mat = self.adjacency_matrix();
        for v in 0..n {
            let mut neighbors = Vec::new();
            for u in 0..n {
                if self.adj(v, u) {
                    neighbors.push(u);
                }
            }
            neighbors.push(v); // include self
            // count edges between all vertices in neighbors
            for i in 0..neighbors.len() {
                for j in i+1..neighbors.len() {
                    if self.adj(neighbors[i], neighbors[j]) {
                        deg2[v] += 1;
                    }
                }
            }
            deg2[v] += degree_factor * (neighbors.len() - 1) as usize; // scale to give more weight to this feature
        }
        deg2
    }

    pub fn degrees3(&self, degree_factor1: usize, degree_factor2: usize) -> Vec<usize> {
        // produces a list, for each vertex of the number of edges in the 2-neighborhood, i.e., 
        // the number of edges adjacent to the vertices and their direct neighbors
        let n = self.num_vertices as usize;
        let mut deg3 = vec![0; n];
        // let mat = self.adjacency_matrix();
        for v in 0..n {
            let mut neighbors3 = FxHashSet::default();
            neighbors3.insert(v);
            for u in 0..n {
                if self.adj(v, u) {
                    neighbors3.insert(u);
                    for w in 0..n {
                        if self.adj(u, w) {
                            neighbors3.insert(w);
                        }
                    }
                }
            }
            // count edges between all vertices in neighbors
            let neighbors: Vec<usize> = neighbors3.iter().cloned().collect();
            for i in 0..neighbors.len() {
                for j in i+1..neighbors.len() {
                    if self.adj(neighbors[i], neighbors[j]) {
                        deg3[v] += 1;
                    }

                }
            }
        }
        let deg2 = self.degrees2(degree_factor1);
        for v in 0..n {
            deg3[v] += degree_factor2 * deg2[v];
        }
        deg3
    }


    /// Returns a vector `res` such that `res[i]` is the number of vertices
    /// at graph distance exactly `i` from vertex `v`.
    pub fn distance_histogram(&self, v: u8) -> Vec<usize> {
    let n = self.num_vertices as usize;
    let mut res = vec![0usize; n + 1];

    let mut seen: u64 = 0;
    let mut frontier: u64 = 1u64 << (v as usize);
    seen |= frontier;
    let mut dist = 0usize;

    while frontier != 0 {
        // count bits in frontier -> number of vertices at distance dist
        res[dist] = frontier.count_ones() as usize;

        // next frontier = neighbors(frontier) & !seen
        let mut nbrs: u64 = 0;
        let mut mm = frontier;
        while mm != 0 {
            let u = mm.trailing_zeros() as usize;
            nbrs |= self.adj[u];
            mm &= mm - 1;
        }
        let next_frontier = nbrs & !seen;
        seen |= next_frontier;
        frontier = next_frontier;
        dist += 1;
    }

    // trim trailing zeros
    res.truncate(dist);
    res
    }

    // pub fn distance_histogram_old(&self, v: u8) -> Vec<usize> {
    //     let n = self.num_vertices as usize;
    //     let mut dist = vec![usize::MAX; n];
    //     let mut res = Vec::new();

    //     let mut queue = std::collections::VecDeque::new();
    //     dist[v as usize] = 0;
    //     queue.push_back(v);

    //     // let adj = self.adjacency_matrix();

    //     while let Some(u) = queue.pop_front() {
    //         let d = dist[u as usize];
    //         for w in 0..n {
    //             if self.adj(u as usize, w) && dist[w] == usize::MAX {
    //                 dist[w] = d + 1;
    //                 queue.push_back(w as u8);
    //             }
    //         }
    //     }

    //     // Count how many vertices at each distance
    //     // let max_d = dist.iter().filter(|&&x| x < usize::MAX).max().copied().unwrap_or(0);
    //     res.resize(self.num_vertices as usize + 1, 0);
    //     for &d in &dist {
    //         if d != usize::MAX {
    //             res[d] += 1;
    //         }
    //     }
    //     res
    // }

    pub fn distance_histogram_keys(&self) -> Vec<usize> {
        let weight_factor = self.num_vertices as usize;
        let mut histograms = Vec::new();
        for v in 0..self.num_vertices {
            let hist = self.distance_histogram(v);
            let mut sum = 0;
            let mut factor = 1;
            for (_, &count) in hist.iter().rev().enumerate() {
                sum += count * factor;
                factor *= weight_factor;
            }
            histograms.push(sum);
        }

        histograms
    }

    pub fn permute(&self, perm: &[u8]) -> Graph {
        let mut edges: Vec<(u8, u8)> = self
            .edges
            .iter()
            .map(|&(u, v)| {
                let (a, b) = (perm[u as usize], perm[v as usize]);
                if a < b { (a, b) } else { (b, a) }
            })
            .collect();
        edges.sort();
        Graph::new(self.num_vertices, edges)
    }

    pub fn to_g6(&self) -> String {
        let graph = self;
        let n = graph.num_vertices;
        assert!(n <= 62, "This encoder only supports graphs with at most 62 vertices.");

        // Write N(n)
        let mut result = String::new();
        result.push((n + 63) as char);

        // Create adjacency bit vector in the correct order
        let mut bitvec = Vec::new();
        for j in 1..n {
            for i in 0..j {
                let bit = graph.edges.contains(&(i, j)) || graph.edges.contains(&(j, i));
                bitvec.push(bit as u8);
            }
        }

        // Pad bitvec with zeros to make length multiple of 6
        while bitvec.len() % 6 != 0 {
            bitvec.push(0);
        }

        // Encode into 6-bit chunks
        for chunk in bitvec.chunks(6) {
            let mut value = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                value |= bit << (5 - i);
            }
            result.push((value + 63) as char);
        }

        result
    }

    // pub fn canonical_label_old(&self, init_colors: Option<&[usize]>) -> (Graph, Vec<u8>) {
    //     let n = self.num_vertices as usize;
    //     let mut classes: Vec<Vec<u8>> = Vec::new();

    //     if let Some(colors) = init_colors {
    //         assert_eq!(colors.len(), n);
    //         use std::collections::HashMap;
    //         let mut map: HashMap<usize, Vec<u8>> = HashMap::new();
    //         for (v, &col) in colors.iter().enumerate() {
    //             map.entry(col).or_default().push(v as u8);
    //         }
    //         let mut keys: Vec<usize> = map.keys().cloned().collect();
    //         keys.sort();
    //         for key in keys {
    //             classes.push(map.remove(&key).unwrap());
    //         }
    //     } else {
    //         // let deg = self.degrees().iter().map(|&d| d as usize).collect::<Vec<usize>>();
    //         // let deg = self.degrees2(100);
    //         // let deg = self.degrees3(100,100);
    //         let deg = self.distance_histogram_keys();
    //         // call this function with the degree partition
    //         return self.canonical_label_old(Some(&deg));

    //         // for v in 0..n {
    //         //     if let Some(pos) = classes.iter().position(|cls| deg[cls[0] as usize] == deg[v]) {
    //         //         classes[pos].push(v as u8);
    //         //     } else {
    //         //         classes.push(vec![v as u8]);
    //         //     }
    //         // }
    //     }

    //     refine(self, &mut classes);

    //     let mut best: Option<(Graph, Vec<u8>)> = None;
    //     let mut perm = vec![0; n];
    //     search(self, &classes, &mut best);
    //     best.unwrap()
    // }

    pub fn canonical_label_bm(&self, init_colors: Option<&[usize]>) -> (Graph, Vec<u8>) {
        let n = self.num_vertices as usize;
        let mut classes: Vec<u64> = Vec::new();

        if let Some(colors) = init_colors {
            assert_eq!(colors.len(), n);
            use std::collections::HashMap;
            let mut map: HashMap<usize, u64> = HashMap::new();
            for (v, &col) in colors.iter().enumerate() {
                *map.entry(col).or_default() |= 1u64 << v;
            }
            let mut keys: Vec<usize> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                classes.push(map.remove(&key).unwrap());
            }
        } else {
            // let deg = self.degrees().iter().map(|&d| d as usize).collect::<Vec<usize>>();
            // let deg = self.degrees2(100);
            // let deg = self.degrees3(100,100);
            let deg = self.distance_histogram_keys();
            // call this function with the degree partition
            return self.canonical_label_bm(Some(&deg));

            // for v in 0..n {
            //     if let Some(pos) = classes.iter().position(|cls| deg[cls[0] as usize] == deg[v]) {
            //         classes[pos].push(v as u8);
            //     } else {
            //         classes.push(vec![v as u8]);
            //     }
            // }
        }

        self.refine(&mut classes);

        let mut best: Option<(Graph, Vec<u8>)> = None;
        search_bm(self, &classes, &mut best);
        best.unwrap()
    }

    pub fn canonical_labels_bm(&self, init_colors: Option<&[usize]>) -> (Graph, Vec<Vec<u8>>) {
        let n = self.num_vertices as usize;
        let mut classes: Vec<u64> = Vec::new();

        if let Some(colors) = init_colors {
            assert_eq!(colors.len(), n);
            use std::collections::HashMap;
            let mut map: HashMap<usize, u64> = HashMap::new();
            for (v, &col) in colors.iter().enumerate() {
                *map.entry(col).or_default() |= 1u64 << v;
            }
            let mut keys: Vec<usize> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                classes.push(map.remove(&key).unwrap());
            }
        } else {
            // let deg = self.degrees().iter().map(|&d| d as usize).collect::<Vec<usize>>();
            // let deg = self.degrees2(100);
            // let deg = self.degrees3(100,100);
            let deg = self.distance_histogram_keys();
            // call this function with the degree partition
            return self.canonical_labels_bm(Some(&deg));

            // for v in 0..n {
            //     if let Some(pos) = classes.iter().position(|cls| deg[cls[0] as usize] == deg[v]) {
            //         classes[pos].push(v as u8);
            //     } else {
            //         classes.push(vec![v as u8]);
            //     }
            // }
        }

        self.refine(&mut classes);

        let mut best: Option<(Graph, Vec<Vec<u8>>)> = None;
        search_multi_bm(self, &classes, &mut best);
        best.unwrap()
    }


    // pub fn canonical_labels(&self, init_colors: Option<&[usize]>) -> (Graph, Vec<Vec<u8>>) {
    //     // The same as canonical label. But returns all permutations that yield the same canonical form.
    //     // The first returned permutation is the one that yields the returned graph
    //     let n = self.num_vertices as usize;
    //     let mut classes: Vec<Vec<u8>> = Vec::new();

    //     if let Some(colors) = init_colors {
    //         assert_eq!(colors.len(), n);
    //         use std::collections::HashMap;
    //         let mut map: HashMap<usize, Vec<u8>> = HashMap::new();
    //         for (v, &col) in colors.iter().enumerate() {
    //             map.entry(col).or_default().push(v as u8);
    //         }
    //         let mut keys: Vec<usize> = map.keys().cloned().collect();
    //         keys.sort();
    //         for key in keys {
    //             classes.push(map.remove(&key).unwrap());
    //         }
    //     } else {
    //         // let deg = self.degrees().iter().map(|&d| d as usize).collect::<Vec<usize>>();
    //         // let deg = self.degrees2(100);
    //         // let deg = self.degrees3(100,100);
    //         let deg = self.distance_histogram_keys();
    //         // call this function with the degree partition
    //         return self.canonical_labels(Some(&deg));

    //         // for v in 0..n {
    //         //     if let Some(pos) = classes.iter().position(|cls| deg[cls[0] as usize] == deg[v]) {
    //         //         classes[pos].push(v as u8);
    //         //     } else {
    //         //         classes.push(vec![v as u8]);
    //         //     }
    //         // }
    //     }

    //     refine(self, &mut classes);

    //     let mut best: Option<(Graph, Vec<Vec<u8>>)> = None;
    //     search_multi(self, &classes, &mut best);
    //     best.unwrap()
    // }


    pub fn automorphisms(&self, init_colors: Option<&[usize]>) -> Vec<Vec<u8>> {
        let (_canon, best_perms) = self.canonical_labels_bm(init_colors);
        if best_perms.is_empty() {
            return vec![];
        }
        let base = &best_perms[0];
        best_perms
            .iter()
            .map(|p| compose(base, &inverse(p)))
            .collect()
    }

    pub fn from_g6(g6: &str) -> Graph {
        let bytes = g6.as_bytes();
        assert!(!bytes.is_empty(), "Empty g6 string");

        // Decode N(n)
        let first = bytes[0];
        assert!(first >= 63, "Invalid graph6 string");

        let n = match first {
            63..=126 => (first - 63) as u8,
            _ => panic!("This decoder only supports n ≤ 62 (1-byte N(n))"),
        };

        // Compute number of bits in the upper triangle: n(n-1)/2
        let num_bits = (n as usize * (n as usize - 1)) / 2;
        let num_bytes = (num_bits + 5) / 6;

        let bit_data = &bytes[1..=num_bytes];
        let mut bits = Vec::with_capacity(num_bits);

        for &byte in bit_data {
            assert!(byte >= 63, "Invalid graph6 data byte");
            let val = byte - 63;
            for i in (0..6).rev() {
                bits.push((val >> i) & 1);
            }
        }

        // Trim any extra bits (if padding was added)
        bits.truncate(num_bits);

        // Reconstruct edge list in order: (0,1), (0,2), (1,2), (0,3), (1,3), (2,3), ...
        let mut edges = Vec::new();
        let mut k = 0;
        for j in 1..n {
            for i in 0..j {
                if bits[k] == 1 {
                    edges.push((i, j));
                }
                k += 1;
            }
        }

        Graph::new(n, edges)
    }
    
    pub fn load_from_file(filename: &str) -> std::io::Result<Vec<String>> {
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);
        // read first line and trsnform to int
        let mut lines = reader.lines();
        let first_line = lines.next().unwrap()?;
        let num_graphs: usize = first_line.trim().parse().unwrap();
        let mut g6_list = Vec::new();
        for line in lines { // .take(num_graphs) {
            let g6 = line?;
            g6_list.push(g6);
        }
        if g6_list.len() != num_graphs {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Number of graphs in file does not match the first line",
            ));
        }
        Ok(g6_list)
    }

    pub fn initial_degree_classes(&self) -> Vec<u64> {
        let g = self;
        // let n = g.num_vertices as usize;
        // let mut classes: Vec<Vec<u8>> = Vec::new();

        use std::collections::HashMap;
        // initial partition: one color per degree
        let mut degree_classes: HashMap<u32, u64> = HashMap::new();
        for v in 0..g.num_vertices as usize {
            let deg = g.adj[v].count_ones();
            *degree_classes.entry(deg).or_default() |= 1u64 << v;
        }
        let initial_classes: Vec<u64> = degree_classes.values().cloned().collect();
        initial_classes
    }


    pub fn load_from_file_nohdr(filename: &str) -> std::io::Result<Vec<String>> {
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);
        // read first line and trsnform to int
        let lines = reader.lines();
        let mut g6_list = Vec::new();
        for line in lines {
            let g6 = line?;
            g6_list.push(g6);
        }
        Ok(g6_list)
    }
    #[inline(always)]
    fn adj_bits(&self, v: usize) -> u64 {
        self.adj[v]
    }

    /// Refine partition of vertices given as vector of bitmasks
    fn refine(&self, classes: &mut Vec<u64>) {
        loop {
            let n_classes = classes.len();
            let mut changed = false;
            let mut new_classes = Vec::with_capacity(n_classes);

            for &mask in classes.iter() {
                // Collect signatures for all vertices in this class
                let mut sig_verts: Vec<(Vec<u8>, u8)> = Vec::new();

                // For each vertex in mask
                let mut mm = mask;
                while mm != 0 {
                    let v = mm.trailing_zeros() as usize;
                    mm &= mm - 1;

                    // Build signature: number of neighbors in each class
                    let mut sig = Vec::with_capacity(n_classes);
                    for &cm in classes.iter() {
                        sig.push((self.adj_bits(v) & cm).count_ones() as u8);
                    }
                    sig_verts.push((sig, v as u8));
                }

                // Sort and group
                sig_verts.sort_by(|(sa, a), (sb, b)| {
                    let c = sa.cmp(sb);
                    if c != std::cmp::Ordering::Equal { c } else { a.cmp(b) }
                });

                // Build new subclasses
                let mut i = 0;
                while i < sig_verts.len() {
                    let mut submask: u64 = 0;
                    let (ref sig0, v0) = sig_verts[i];
                    submask |= 1u64 << (v0 as usize);
                    i += 1;
                    while i < sig_verts.len() && sig_verts[i].0 == *sig0 {
                        submask |= 1u64 << (sig_verts[i].1 as usize);
                        i += 1;
                    }
                    new_classes.push(submask);
                }

                if new_classes.len() > classes.len() {
                    changed = true;
                }
            }

            *classes = new_classes;
            if !changed {
                break;
            }
        }
    }

}

// fn classes_to_bitmasks(classes: &Vec<Vec<u8>>, n: usize) -> Vec<u64> {
//     let mut masks = Vec::with_capacity(classes.len());
//     for cls in classes.iter() {
//         let mut mask: u64 = 0;
//         for &v in cls.iter() {
//             mask |= 1u64 << (v as usize);
//         }
//         masks.push(mask);
//     }
//     masks
// }

// fn bitmasks_to_classes(bitmasks: &Vec<u64>, n: usize) -> Vec<Vec<u8>> {
//     let mut classes = Vec::with_capacity(bitmasks.len());
//     for &mask in bitmasks.iter() {
//         let mut cls = Vec::new();
//         let mut mm = mask;
//         while mm != 0 {
//             let v = mm.trailing_zeros() as usize;
//             cls.push(v as u8);
//             mm &= mm - 1;
//         }
//         classes.push(cls);
//     }
//     classes
// }



// fn refine_old(g: &Graph, classes: &mut Vec<Vec<u8>>) {
//     // println!("Initial classes: {:?}", classes);
//     // let mat = g.adjacency_matrix();

//     loop {
//         let mut changed = false;
//         let mut new_classes: Vec<Vec<u8>> = Vec::new();

//         for cls in classes.iter() {
//             let mut buckets: FxHashMap<Vec<usize>, Vec<u8>> = FxHashMap::default();
//             for &v in cls {
//                 let mut signature = Vec::new();
//                 for other in classes.iter() {
//                     let count = other.iter().filter(|&&u| g.adj(v as usize, u as usize)).count();
//                     signature.push(count);
//                 }
//                 buckets.entry(signature).or_default().push(v);
//             }
//             let mut bucket_items: Vec<_> = buckets.into_iter().collect();
//             bucket_items.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
//             for (_, vals) in bucket_items {
//                 new_classes.push(vals);
//             }
//             if new_classes.len() > classes.len() {
//                 changed = true;
//             }
//         }
//         if !changed {
//             break;
//         } else {
//             *classes = new_classes;
//         }
//     }
//     // println!("Refined classes: {:?}", classes);
// }

// fn search(
//     g: &Graph,
//     classes: &Vec<Vec<u8>>,
//     best: &mut Option<(Graph, Vec<u8>)>,
// ) {
//     // let mut mybest = if let Some((best_graph, pp)) = best {
//     //     Some((best_graph.clone(), vec![pp.clone()]))
//     // } else {
//     //     None
//     // };
//     // let xx = search_multi(g, classes, &mut mybest);
//     // if let Some((best_graph, pp)) = mybest {
//     //     *best = Some((best_graph, pp[0].clone()));
//     // }
//     // return;
//     let mut perm = vec![0; g.num_vertices as usize];

//     // let mut tclasses = vec![vec![]; g.num_vertices as usize];
//     // for i in 0..(g.num_vertices as usize) {
//     //     tclasses[i].push(i as u8);
//     // }
//     // let classes = &tclasses;


//     if classes.iter().all(|cls| cls.len() == 1) {
//         // we found a leaf
//         let mut idx = 0;
//         for cls in classes {
//             for &v in cls {
//                 perm[v as usize] = idx as u8;
//                 idx += 1;
//             }
//         }
//         let g_perm = g.permute(&perm);
//         if let Some((best_graph, _)) = best {
//             if g_perm.edges < best_graph.edges {
//                 *best = Some((g_perm, perm.clone()));
//             }
//         } else {
//             *best = Some((g_perm, perm.clone()));
//         }
//         return;
//     }

//     let class_pos = classes.iter().position(|cls| cls.len() > 1).unwrap();
//     let class = &classes[class_pos];

//     for &v in class {
//         // print!(".");
//         let mut new_classes = Vec::new();
//         for (i, cls) in classes.iter().enumerate() {
//             if i == class_pos {
//                 let mut others: Vec<u8> = cls.iter().cloned().filter(|&x| x != v).collect();
//                 if !others.is_empty() {
//                     new_classes.push(others);
//                 }
//                 new_classes.push(vec![v]);
//             } else {
//                 new_classes.push(cls.clone());
//             }
//         }

//         let mut refined = new_classes.clone();
//         refine(g, &mut refined);
//         search(g, &refined, best);
//     }
// }

fn search_bm(
    g: &Graph,
    classes: &Vec<u64>,
    best: &mut Option<(Graph, Vec<u8>)>,
) {
    // let mut mybest = if let Some((best_graph, pp)) = best {
    //     Some((best_graph.clone(), vec![pp.clone()]))
    // } else {
    //     None
    // };
    // let xx = search_multi(g, classes, &mut mybest);
    // if let Some((best_graph, pp)) = mybest {
    //     *best = Some((best_graph, pp[0].clone()));
    // }
    // return;
    let mut perm = vec![0; g.num_vertices as usize];

    // let mut tclasses = vec![vec![]; g.num_vertices as usize];
    // for i in 0..(g.num_vertices as usize) {
    //     tclasses[i].push(i as u8);
    // }
    // let classes = &tclasses;


    if classes.iter().all(|cls| cls.count_ones() == 1) {
        // we found a leaf
        let mut idx = 0;
        for cls in classes {
            let v_idx = cls.trailing_zeros() as u8;
            perm[v_idx as usize] = idx as u8;
            idx += 1;
        }
        let g_perm = g.permute(&perm);
        if let Some((best_graph, _)) = best {
            if g_perm.edges < best_graph.edges {
                *best = Some((g_perm, perm.clone()));
            }
        } else {
            *best = Some((g_perm, perm.clone()));
        }
        return;
    }

    let class_pos = classes.iter().position(|cls| cls.count_ones() > 1).unwrap();
    let class = &classes[class_pos];

    for v in 0..g.num_vertices {
        if (class & (1u64 << v)) == 0 {
            continue;
        }
        let v = v as u8;
        // print!(".");
        let mut new_classes = Vec::new();
        for (i, cls) in classes.iter().enumerate() {
            if i == class_pos {
                let others = cls & !(1u64 << v);
                if others != 0 {
                    new_classes.push(others);
                }
                new_classes.push(1u64 << v);
            } else {
                new_classes.push(*cls);
            }
        }

        let mut refined = new_classes.clone();
        g.refine(&mut refined);
        search_bm(g, &refined, best);
    } 
    // {
    //     // print!(".");
    //     let mut new_classes = Vec::new();
    //     for (i, cls) in classes.iter().enumerate() {
    //         if i == class_pos {
    //             let mut others: Vec<u8> = cls.iter().cloned().filter(|&x| x != v).collect();
    //             if !others.is_empty() {
    //                 new_classes.push(others);
    //             }
    //             new_classes.push(vec![v]);
    //         } else {
    //             new_classes.push(cls.clone());
    //         }
    //     }

    //     let mut refined = new_classes.clone();
    //     refine(g, &mut refined);
    //     search(g, &refined, best);
    // }
}

fn search_multi_bm(
    g: &Graph,
    classes: &Vec<u64>,
    best: &mut Option<(Graph, Vec<Vec<u8>>)>,
) {
    // let mut mybest = if let Some((best_graph, pp)) = best {
    //     Some((best_graph.clone(), vec![pp.clone()]))
    // } else {
    //     None
    // };
    // let xx = search_multi(g, classes, &mut mybest);
    // if let Some((best_graph, pp)) = mybest {
    //     *best = Some((best_graph, pp[0].clone()));
    // }
    // return;
    let mut perm = vec![0; g.num_vertices as usize];

    // let mut tclasses = vec![vec![]; g.num_vertices as usize];
    // for i in 0..(g.num_vertices as usize) {
    //     tclasses[i].push(i as u8);
    // }
    // let classes = &tclasses;


    if classes.iter().all(|cls| cls.count_ones() == 1) {
        // we found a leaf
        let mut idx = 0;
        for cls in classes {
            let v_idx = cls.trailing_zeros() as u8;
            perm[v_idx as usize] = idx as u8;
            idx += 1;
        }
        let g_perm = g.permute(&perm);
        if let Some((best_graph, _)) = best {
            if g_perm.edges < best_graph.edges {
                *best = Some((g_perm, vec![perm.clone()]));
            } else if g_perm.edges == best_graph.edges {
                if let Some((_, perms)) = best.as_mut() {
                    perms.push(perm.clone());
                }
            }
        } else {
            *best = Some((g_perm, vec![perm.clone()]));
        }
        return;
    }

    let class_pos = classes.iter().position(|cls| cls.count_ones() > 1).unwrap();
    let class = &classes[class_pos];

    for v in 0..g.num_vertices {
        if (class & (1u64 << v)) == 0 {
            continue;
        }
        let v = v as u8;
        // print!(".");
        let mut new_classes = Vec::new();
        for (i, cls) in classes.iter().enumerate() {
            if i == class_pos {
                let others = cls & !(1u64 << v);
                if others != 0 {
                    new_classes.push(others);
                }
                new_classes.push(1u64 << v);
            } else {
                new_classes.push(*cls);
            }
        }

        let mut refined = new_classes.clone();
        g.refine(&mut refined);
        search_multi_bm(g, &refined, best);
    } 
    // {
    //     // print!(".");
    //     let mut new_classes = Vec::new();
    //     for (i, cls) in classes.iter().enumerate() {
    //         if i == class_pos {
    //             let mut others: Vec<u8> = cls.iter().cloned().filter(|&x| x != v).collect();
    //             if !others.is_empty() {
    //                 new_classes.push(others);
    //             }
    //             new_classes.push(vec![v]);
    //         } else {
    //             new_classes.push(cls.clone());
    //         }
    //     }

    //     let mut refined = new_classes.clone();
    //     refine(g, &mut refined);
    //     search(g, &refined, best);
    // }
}

// fn search_multi(
//     g: &Graph,
//     classes: &Vec<Vec<u8>>,
//     // perm: &mut Vec<u8>,
//     best: &mut Option<(Graph, Vec<Vec<u8>>)>,
// ) {
//     if classes.iter().all(|cls| cls.len() == 1) {
//         // we found a leaf
//         let mut idx = 0;
//         let mut perm = vec![0; g.num_vertices as usize];
//         for cls in classes {
//             for &v in cls {
//                 perm[v as usize] = idx as u8;
//                 idx += 1;
//             }
//         }
//         let g_perm = g.permute(&perm);
//         if let Some((best_graph, perms)) = best.as_mut() {
//             if g_perm.edges < best_graph.edges {
//                 *best = Some((g_perm, vec![perm.clone()]));
//             } else if g_perm.edges == best_graph.edges {
//                 perms.push(perm.clone());
//             }
//         } else {
//             *best = Some((g_perm, vec![perm.clone()]));
//         }
//         return;
//     }

//     let class_pos = classes.iter().position(|cls| cls.len() > 1).unwrap();
//     let class = &classes[class_pos];

//     for &v in class {
//         let mut new_classes = Vec::new();
//         for (i, cls) in classes.iter().enumerate() {
//             if i == class_pos {
//                 let mut others: Vec<u8> = cls.iter().cloned().filter(|&x| x != v).collect();
//                 if !others.is_empty() {
//                     new_classes.push(others);
//                 }
//                 new_classes.push(vec![v]);
//             } else {
//                 new_classes.push(cls.clone());
//             }
//         }

//         let mut refined = new_classes.clone();
//         refine(g, &mut refined);
//         search_multi(g, &refined, best);
//     }
// }

// fn random_graph<R: Rng>(rng: &mut R, n: u8, edge_prob: f64) -> Graph {
//     let mut edges = Vec::new();
//     for i in 0..n {
//         for j in i+1..n {
//             if rng.gen_bool(edge_prob) {
//                 edges.push((i, j));
//             }
//         }
//     }
//     Graph::new(n, edges)
// }

fn random_permutation<R: Rng>(rng: &mut R, n: u8) -> Vec<u8> {
    let mut perm: Vec<u8> = (0..n).collect();
    perm.shuffle(rng);
    perm
}

// fn test_canonical_label_random(n_tests: usize) {
//     let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
//     let n = 8;
//     let mut mismatches = 0;
//     let start_total = Instant::now();

//     for i in 0..n_tests {
//         let g = random_graph(&mut rng, n, 0.3);
//         // let g = Graph::from_g6("GAl??G");
//         // println!("Graph A: {} ", g.to_g6());
//         let perm = random_permutation(&mut rng, n);
//         let g2 = g.permute(&perm);
//         // let g2 = Graph::from_g6("GSWOO?");
//         // println!("Graph B: {} ", g2.to_g6());

//         let start = Instant::now();
//         let (can1, _) = g.canonical_label_bm(None);
//         // println!("Canonical A: {} ", can1.to_g6());
//         let (can2, _) = g2.canonical_label_bm(None);
//         let duration = start.elapsed();
//         // println!("Duration: {:?} ms", duration.as_secs_f64() * 1e3);
//         // println!("Canonical B: {} ", can2.to_g6());

//         // println!("All: \n{}\n{}\n{}\n{}", g.to_g6(), g2.to_g6(), can1.to_g6(), can2.to_g6());

//         if can1.edges != can2.edges {
//             mismatches += 1;
//             println!("❌ Mismatch at test {i}");
//         }

//         println!("Test {i}: {:?} ms", duration.as_secs_f64() * 1e3);
//     }

//     let total_time = start_total.elapsed();
//     println!("---");
//     println!("Tests run: {}", n_tests);
//     println!("Mismatches: {}", mismatches);
//     println!("Total time: {:.3} s", total_time.as_secs_f64());
//     println!("Avg per graph: {:.3} ms", total_time.as_secs_f64() * 1e3 / n_tests as f64);
// }

fn test_canonical_label_file(filename: &str, max_ntests: usize) {
    let graphs = Graph::load_from_file(filename).unwrap().iter().map(|g6| Graph::from_g6(g6)).collect::<Vec<Graph>>();
    let mut n_tests = graphs.len();
    println!("Loaded {} graphs from file {}", n_tests, filename);
    if max_ntests > 0 && n_tests > max_ntests {
        n_tests = max_ntests;
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let n = graphs[0].num_vertices;
    let mut mismatches = 0;
    let start_total = Instant::now();
    let mut with_autos =0;

    for i in 0..n_tests {
        let g = &graphs[i];
        // let g = Graph::from_g6("GAl??G");
        // println!("Graph A: {} ", g.to_g6());
        let perm = random_permutation(&mut rng, n);
        let g2 = g.permute(&perm);
        let autos = g.automorphisms(None);
        if autos.len() > 1 {
            // print!("{}", autos.len());
            with_autos += 1;
        }
        // let g2 = Graph::from_g6("GSWOO?");
        // println!("Graph B: {} ", g2.to_g6());

        // let start = Instant::now();
        let (can1, _) = g.canonical_label_bm(None);
        // let (can1, _) = g.canonical_label(g.initial_degree_classes());
        // println!("Canonical A: {} ", can1.to_g6());
        let (can2, _) = g2.canonical_label_bm(None);
        // let (can2, _) = g2.canonical_label(g2.initial_degree_classes());
        // let duration = start.elapsed();
        // println!("Duration: {:?} ms", duration.as_secs_f64() * 1e3);
        // println!("Canonical B: {} ", can2.to_g6());

        // println!("All: \n{}\n{}\n{}\n{}", g.to_g6(), g2.to_g6(), can1.to_g6(), can2.to_g6());

        if can1.edges != can2.edges {
            mismatches += 1;
            println!("❌ Mismatch at test {i}");
        }

        // println!("Test {i}: {:?} ms", duration.as_secs_f64() * 1e3);
    }

    let total_time = start_total.elapsed();
    println!("---");
    println!("Tests run: {}", n_tests);
    println!("Mismatches: {}", mismatches);
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!("Avg per graph: {:.3} ms", total_time.as_secs_f64() * 1e3 / n_tests as f64);
    println!("Graphs with nontrivial automorphisms: {}", with_autos);
}


fn main() {
    // test_canonical_label_random(20);
    test_canonical_label_file("data/graphs10_0.g6",1000);
    test_canonical_label_file("data/graphs10_1.g6",1000);
    // test_canonical_label_file("data/graphs11_2.g6",1000);
    // test_canonical_label_file("data/graphs9_3.g6",1000);
}
