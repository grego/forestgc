use std::collections::{BTreeMap, HashSet};
use std::io::BufRead;

type HashType = usize;

type GraphScore = Vec<u64>;

#[derive(Clone, Debug)]
pub struct Graph {
    pub num_vertices: u8,
    pub edges: Vec<(u8, u8)>,
    pub adj: Vec<u64>, // adjacency matrix as bit-packed rows
}

/// An iterator over the position of bits in a bitmask.
pub struct BitMask(pub u64);

/// Calculate the inverse of the provided permutation.
pub fn inverse(perm: &[u8]) -> Vec<u8> {
    let mut inv = vec![0u8; perm.len()];
    for (i, &p) in perm.iter().enumerate() {
        inv[p as usize] = i as u8;
    }
    inv
}

/// Calculate the composition of two permutations.
pub fn compose(a: &[u8], b: &[u8]) -> Vec<u8> {
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

    /// Returns a vector `res` such that `res[i]` is the number of vertices
    /// at graph distance exactly `i` from vertex `v`.
    pub fn distance_histogram(&self, v: u8) -> Vec<usize> {
        let n = self.num_vertices as usize;
        let mut res = Vec::with_capacity(n);

        let mut seen: u64 = 0;
        let mut frontier: u64 = 1u64 << (v as usize);
        seen |= frontier;

        while frontier != 0 {
            // count bits in frontier -> number of vertices at distance dist
            res.push(frontier.count_ones() as usize);

            // next frontier = neighbors(frontier) & !seen
            let mut nbrs: u64 = 0;
            for u in BitMask(frontier) {
                nbrs |= self.adj[u];
            }
            frontier = nbrs & !seen;
            seen |= frontier;
        }
        res
    }

    pub fn distance_histogram_keys(&self) -> Vec<usize> {
        let weight_factor = self.num_vertices as usize;
        let mut histograms = Vec::with_capacity(self.num_vertices as usize);
        for v in 0..self.num_vertices {
            let hist = self.distance_histogram(v);
            let mut sum = 0;
            let mut factor = 1;
            for &count in hist.iter().rev() {
                sum += count * factor;
                factor *= weight_factor;
            }
            histograms.push(sum);
        }

        histograms
    }

    /// Permute the graph with the provided permutation.
    pub fn permute(&self, perm: &[u8]) -> Graph {
        Graph::new(self.num_vertices, permute_edges(&self.edges, perm))
    }

    pub fn to_g6(&self) -> String {
        let graph = self;
        let n = graph.num_vertices;
        assert!(
            n <= 62,
            "This encoder only supports graphs with at most 62 vertices."
        );

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

    #[inline(always)]
    pub fn canonical_label(&self) -> (Graph, Vec<u8>) {
        let (g, pp) = self.canonical_labels();
        (g, pp[0].clone())
    }

    #[inline(always)]
    pub fn canonical_label_col(&self, init_colors: &[usize]) -> (Graph, Vec<u8>) {
        let (g, pp) = self.canonical_labels_col(init_colors);
        (g, pp[0].clone())
    }

    #[inline(always)]
    pub fn canonical_labels(&self) -> (Graph, Vec<Vec<u8>>) {
        let zero_colors = vec![0usize; self.num_vertices as usize];
        self.canonical_labels_col(&zero_colors)
    }

    pub fn canonical_labels_col(&self, init_colors: &[usize]) -> (Graph, Vec<Vec<u8>>) {
        let n = self.num_vertices as usize;
        let mut classes: Vec<u64> = vec![(1 << n) - 1]; // start with one big class
        // let start = Instant::now();
        assert_eq!(init_colors.len(), n);
        // get some initial coloring by applying relatively strong vertex invariants
        classes = self.refined_coloring(&classes, init_colors);

        let hash = self.distance_histogram_keys();
        classes = self.refined_coloring(&classes, &hash);

        self.refine(&mut classes);
        // let hash = self.myhash(&classes);
        // classes = self.refined_coloring(&classes, &hash);
        // let elapsed = start.elapsed();
        // println!("initial coloring took {:.6} ms", elapsed.as_secs_f64() * 1e3);

        let mut best: Option<(GraphScore, Vec<Vec<u8>>)> = None;
        self.search_multi_bm(&classes, &mut best);
        // let elapsed2 = start.elapsed();
        // println!(
        //     "search_multi_bm took {:.6} ms",
        //     elapsed2.as_secs_f64() * 1e3
        // );

        // display timing only if one took more than .01ms
        let (_, perms) = best.unwrap();
        // if elapsed.as_secs_f64() * 1e3 > 0.01 || elapsed2.as_secs_f64() * 1e3 > 0.2 {
        //     let n_autos = perms.len();
        //     println!(
        //         "Refinement took {:.6} ms, search took {:.6} ms, {n_autos} automorphisms",
        //         elapsed.as_secs_f64() * 1e3,
        //         elapsed2.as_secs_f64() * 1e3
        //     );
        // }

        let gcanon = self.permute(&perms[0]);
        (gcanon, perms)
    }

    #[inline(always)]
    pub fn automorphisms(&self) -> Vec<Vec<u8>> {
        let zero_colors = vec![0usize; self.num_vertices as usize];
        self.automorphisms_col(&zero_colors)
    }

    pub fn automorphisms_col(&self, init_colors: &[usize]) -> Vec<Vec<u8>> {
        let (_canon, best_perms) = self.canonical_labels_col(init_colors);
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
            63..=126 => first - 63,
            _ => panic!("This decoder only supports n ≤ 62 (1-byte N(n))"),
        };

        // Compute number of bits in the upper triangle: n(n-1)/2
        let num_bits = (n as usize * (n as usize - 1)) / 2;
        let num_bytes = num_bits.div_ceil(6);

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
        reader.lines().collect()
    }

    /// Refines a given original coloring based on given hash values provided for every vertex.
    /// Each of the original classes is (possibly) split into multiple classes of vertices of equal hash values.
    /// The new subclasses are sorted by hash value.
    #[inline(always)]
    fn refined_coloring(&self, orig_classes: &[u64], hashes: &[HashType]) -> Vec<u64> {
        let n = self.num_vertices as usize;
        let mut new_classes = Vec::with_capacity(n);
        for &class_mask in orig_classes.iter().filter(|c| **c != 0) {
            // Map from hash value to bitmask of vertices in this class with that hash
            let mut hash_map: BTreeMap<HashType, u64> = BTreeMap::new();
            for v in BitMask(class_mask) {
                let h = hashes[v];
                *hash_map.entry(h).or_default() |= 1u64 << v;
            }
            for (_h, mask) in hash_map {
                new_classes.push(mask);
            }
        }
        new_classes
    }

    /// Refines a partition of vertices (given as bitmask vector) using adjacency information.
    /// Uses integer hashes instead of Vec<u8> signamarch nativetures for speed.
    fn refine(&self, classes: &mut Vec<u64>) {
        loop {
            let mut changed = false;

            // For each class, split by neighborhood signatures
            for i in 0..classes.len() {
                let class_mask = classes[i];
                if class_mask.count_ones() <= 1 {
                    continue;
                }

                // Compute integer signature for each vertex in this class
                // let mut sigs: Vec<(usize, u8)> = Vec::new();
                let mut hashes: Vec<(HashType, u64)> = Vec::with_capacity(4);
                for v in BitMask(class_mask) {
                    // Compute hash signature based on neighbor counts in each class
                    let mut h: usize = 0;
                    for (j, &cm) in classes.iter().enumerate() {
                        let cnt = (self.adj[v] & cm).count_ones() as usize;
                        // Simple multiplicative hash; 257 is small prime
                        h = h.wrapping_mul(257).wrapping_add(cnt + j * 17);
                    }
                    let p = hashes.iter().position(|(x, _)| *x == h).unwrap_or_else(|| {
                        let len = hashes.len();
                        hashes.push((h, 0));
                        len
                    });
                    hashes[p].1 |= 1u64 << v;
                }
                if hashes.len() > 1 {
                    hashes.sort_unstable_by_key(|(h, _)| *h);
                    // Replace class i by the new parts
                    classes.remove(i);
                    for (k, (_, part)) in hashes.into_iter().enumerate() {
                        classes.insert(i + k, part);
                    }
                    changed = true;
                    break; // restart refinement because indices changed
                }
            }

            if !changed {
                break;
            }
        }
    }

    #[inline(always)]
    fn graph_score(&self, perm: &[u8]) -> GraphScore {
        let n = self.num_vertices as usize;
        let mut adj = vec![0u64; n];
        for &(u, v) in &self.edges {
            let a = perm[u as usize] as usize;
            let b = perm[v as usize] as usize;
            adj[a] |= 1u64 << b;
            adj[b] |= 1u64 << a;
        }
        adj
    }

    fn search_multi_bm(&self, classes: &[u64], best: &mut Option<(GraphScore, Vec<Vec<u8>>)>) {
        let Some(class_pos) = classes.iter().position(|cls| cls.count_ones() > 1) else {
            // we found a leaf
            let mut perm = vec![0; self.num_vertices as usize];
            for (i, cls) in classes.iter().enumerate() {
                let v_idx = cls.trailing_zeros() as u8;
                perm[v_idx as usize] = i as u8;
            }
            let gperm_score = self.graph_score(&perm);
            if let Some((best_graph_score, perms)) = best {
                //let best_bitstr = best_graph.bitstring();
                // let g_bitstr = g_perm.bitstring();
                if gperm_score <= *best_graph_score {
                    if gperm_score < *best_graph_score {
                        *best_graph_score = gperm_score;
                        perms.clear()
                    }
                    perms.push(perm.clone());
                }
            } else {
                *best = Some((gperm_score, vec![perm.clone()]));
            }
            return;
        };
        let class = &classes[class_pos];

        for v in 0..self.num_vertices {
            if (class & (1u64 << v)) == 0 {
                continue;
            }
            let mut new_classes = Vec::with_capacity(classes.len() + 1);
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

            let mut refined = new_classes;
            self.refine(&mut refined);

            self.search_multi_bm(&refined, best);
        }
    }

    /// Turn valency 2 vertices into new edges.
    pub fn simplify(&self) -> Self {
        let mut new_edges = HashSet::new();
        let mut perm: Vec<_> = (0..self.num_vertices).collect();
        let mut new_v = 0;
        for (i, &a) in self.adj.iter().enumerate() {
            match a.count_ones() {
                1 => {}
                2 => {
                    let mut iter = BitMask(a);
                    let v = iter.next().unwrap();
                    let w = iter.next().unwrap();
                    new_edges.insert((v as u8, w as u8));
                }
                _ => {
                    perm[i] = new_v;
                    new_v += 1;
                }
            }
        }
        let new_edges: Vec<_> = new_edges.drain().collect();
        Graph::new(new_v, permute_edges(&new_edges, &perm))
    }

    pub fn to_multigraph(&self) -> Self {
        let mut new_edges = Vec::new();
        let mut perm: Vec<_> = (0..self.num_vertices).collect();
        let mut new_v = 0;
        for (i, &a) in self.adj.iter().enumerate() {
            match a.count_ones() {
                1 => {}
                2 => {
                    let mut iter = BitMask(a);
                    let v = iter.next().unwrap();
                    let w = iter.next().unwrap();
                    new_edges.push((v as u8, w as u8));
                }
                _ => {
                    perm[i] = new_v;
                    new_v += 1;
                }
            }
        }
        Graph::new(new_v, permute_edges(&new_edges, &perm))
    }

    pub fn to_dot(&self) -> String {
        let mut s = "graph {\n".to_string();
        for &(v, w) in &self.edges {
            s.push_str(&format!("{v} -- {w}\n"));
        }
        s.push_str("}");
        s
    }
}

pub fn permute_edges(edges: &[(u8, u8)], perm: &[u8]) -> Vec<(u8, u8)> {
    let mut edges: Vec<(u8, u8)> = edges
        .iter()
        .map(|&(u, v)| {
            let (a, b) = (perm[u as usize], perm[v as usize]);
            if a < b { (a, b) } else { (b, a) }
        })
        .collect();
    edges.sort_unstable();
    edges
}

impl Iterator for BitMask {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let BitMask(mask) = *self;
        if mask == 0 {
            return None;
        }
        let u = mask.trailing_zeros() as usize;
        *self = BitMask(mask & (mask - 1));
        Some(u)
    }
}
