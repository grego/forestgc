use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;
use std::mem;

type HashType = u128;

type GraphScore = Vec<u64>;

type Edge = (u8, u8);

#[derive(Clone, Debug)]
pub struct Graph {
    pub num_vertices: u8,
    pub edges: Vec<Edge>,
    pub adj: Vec<u64>, // adjacency matrix as bit-packed rows
}

#[derive(Clone, Debug)]
pub struct ForestedGraph {
    pub graph: Graph,
    pub forest: Vec<u8>,
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

/// Compute the sign of the permutation on the provided subset.
/// The caller must ensure that the subset is ordered and perm remains a bijection
/// when restricted to it.
pub fn sign_subset<
    I: IntoIterator<Item = J, IntoIter = K>,
    J: Into<usize>,
    K: Iterator<Item = J> + Clone,
>(
    perm: &[u8],
    subset: I,
) -> i8 {
    let mut subset = subset.into_iter();
    let mut sign = 1;
    while let Some(i) = subset.next() {
        let ss = subset.clone();
        let i = i.into();
        for j in ss {
            let j = j.into();
            if perm[i] > perm[j] {
                sign *= -1;
            }
        }
    }
    sign
}

/// Compute the sign of a permutation
pub fn sign(perm: &[u8]) -> i8 {
    sign_subset(perm, 1..perm.len())
}

/// Calculate the composition of two permutations.
pub fn compose(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().map(|&x| b[x as usize]).collect()
}

impl ForestedGraph {
    pub fn new(graph: Graph, forest: Vec<u8>) -> Self {
        ForestedGraph {
            graph,
            forest,
        }
    }
    pub fn contract_edge(&self, v: u8) -> ForestedGraph {
        let mut g = self.graph.clone();
        let mut f = self.forest.clone();
        g.contract_binary_neighborhood(v);
        let pos = f.iter().position(|&x| x == v);
        if !pos.is_none() {
            f.remove(pos.unwrap());
            f.iter_mut().for_each(|x| if *x > v { *x -= 1 });
        }
        ForestedGraph::new(g,f)
    }
    pub fn forget_forest_edge(&self, v:u8) -> ForestedGraph {
        let mut f = self.forest.clone();
        let pos = f.iter().position(|&x| x == v);
        f.remove(pos.unwrap());
        ForestedGraph::new(self.graph.clone(),f)
    }
}

impl Graph {
    pub fn new(num_vertices: u8, mut edges: Vec<(u8, u8)>) -> Self {
        edges.sort_unstable();
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

    pub fn distance_histogram_keys(&self) -> Vec<u128> {
        let weight_factor = self.num_vertices as u128;
        let mut histograms = Vec::with_capacity(self.num_vertices as usize);
        for v in 0..self.num_vertices {
            let hist = self.distance_histogram(v);
            let mut sum: u128 = 0;
            let mut factor: u128 = 1;
            for &count in hist.iter().rev() {
                sum += count as u128 * factor;
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
    pub fn canonical_label_col(&self, init_colors: &[u128]) -> (Graph, Vec<u8>) {
        let (g, pp) = self.canonical_labels_col(init_colors);
        (g, pp[0].clone())
    }

    #[inline(always)]
    pub fn canonical_labels(&self) -> (Graph, Vec<Vec<u8>>) {
        let zero_colors = vec![0u128; self.num_vertices as usize];
        self.canonical_labels_col(&zero_colors)
    }

    pub fn canonical_labels_col(&self, init_colors: &[u128]) -> (Graph, Vec<Vec<u8>>) {
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
        let zero_colors = vec![0u128; self.num_vertices as usize];
        self.automorphisms_col(&zero_colors)
    }

    pub fn automorphisms_col(&self, init_colors: &[u128]) -> Vec<Vec<u8>> {
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
                    let mut h: u128 = 0;
                    for (j, &cm) in classes.iter().enumerate() {
                        let cnt = (self.adj[v] & cm).count_ones() as u128;
                        // Simple multiplicative hash; 257 is small prime
                        h = h.wrapping_mul(257).wrapping_add(cnt + j as u128 * 17);
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

    /// Add a new unary vertex connected to the provided vertex.
    pub fn add_unary_vertex(&mut self, v: u8) {
        let new = self.num_vertices;
        self.num_vertices += 1;
        self.edges.push((v, new));
        self.adj[v as usize] |= 1 << new;
        self.adj.push(1 << v);
    }

    /// Delete the vertex with the given index.
    pub fn delete_vertex(&mut self, v: u8) {
        self.num_vertices -= 1;
        let dec = |w| if w < v { w } else { w - 1 };
        let edges = mem::take(&mut self.edges);
        self.edges = edges
            .into_iter()
            .filter(|(u, w)| *u == v || *w == v)
            .map(|(u, w)| (dec(u), dec(w)))
            .collect();
        self.adj.remove(v as usize);
        let mask = (1 << v) - 1;
        for a in self.adj.iter_mut() {
            *a = (*a & mask) | ((*a >> 1) & !mask);
        }
    }

    /// Contract the neigborhood of a binary vertex.
    pub fn contract_binary_neighborhood(&mut self, mut v: u8) {
        self.num_vertices -= 1;
        let mut iter = BitMask(self.adj[v as usize]);
        let u = iter.next().unwrap() as u8;
        let mut w = iter.next().unwrap() as u8;
        let f = |a| match a {
            a if a == v => None,
            a if a == w => Some(u),
            _ => Some(a - (a > v) as u8 - (a > w) as u8),
        };
        let edges = mem::take(&mut self.edges);
        self.edges = edges
            .into_iter()
            .filter_map(|(a, b)| Some((f(a)?, f(b)?)))
            .collect();
        if v > w {
            mem::swap(&mut v, &mut w);
        }
        for a in self.adj.iter_mut() {
            *a = delete_vertex_from_mask(v, *a);
            *a = delete_vertex_from_mask(w - 1, *a);
        }
    }

    /// Turn valency 2 vertices into new edges.
    /// Return the list of original indices of the new edges.
    pub fn simplify(&self, retain_multiedges: bool) -> (Self, Vec<(Edge, u8)>) {
        let mut new_edges = HashMap::new();
        let mut perm: Vec<_> = (0..self.num_vertices).collect();
        let mut new_v = 0;
        for (i, &a) in self.adj.iter().enumerate() {
            match a.count_ones() {
                0..=1 => {}
                2 => {
                    let mut iter = BitMask(a);
                    let v = iter.next().unwrap();
                    let w = iter.next().unwrap();
                    let e = new_edges.entry((v as u8, w as u8)).or_insert((i as u8, 0));
                    e.1 += 1;
                }
                _ => {
                    perm[i] = new_v;
                    new_v += 1;
                }
            }
        }
        let new_edges: Vec<_> = new_edges
            .drain()
            .filter_map(|(e, (i, c))| Some((e, i)).filter(|_| retain_multiedges || c == 1))
            .collect();
        let mut perm_edges = permute_indexed_edges(&new_edges, &perm);
        perm_edges.sort_unstable_by_key(|(e, _)| *e);
        (
            Graph::new(new_v, perm_edges.iter().cloned().map(|(e, _)| e).collect()),
            perm_edges,
        )
    }

    pub fn subforests(&self, min_edges: usize, max_edges: usize) -> Vec<Vec<Edge>> {
        // The currently found forest.
        let mut forest = Vec::with_capacity(self.num_vertices as usize - 1);
        // For each vertex, the smallest number of a vertex in its component
        // in the currently found forest.
        let mut components: Vec<_> = (0..self.num_vertices).collect();
        // Bitmasks of the component of the current vertex.
        let mut component_masks: Vec<_> = (0..self.num_vertices).map(|i| 1_u64 << i).collect();
        let mut stack: Vec<(_, _, _, u64)> = Vec::with_capacity(self.num_vertices as usize - 1);
        let mut output = Vec::new();

        let mut i = 0;
        loop {
            if forest.len() == max_edges || i == self.edges.len() {
                let Some((j, cv, cw, mask)) = stack.pop() else {
                    break;
                };
                forest.pop();
                component_masks[cv as usize] &= !mask;
                for u in BitMask(mask) {
                    components[u] = cw;
                }
                i = j + 1;
                continue;
            }

            let (v, w) = self.edges[i];
            let (mut cv, mut cw) = (components[v as usize], components[w as usize]);
            if cv == cw {
                i += 1;
                continue;
            } else if cw < cv {
                mem::swap(&mut cv, &mut cw);
                //mem::swap(&mut v, &mut w);
            }
            let mask = component_masks[cw as usize];
            stack.push((i, cv, cw, mask));
            component_masks[cv as usize] |= mask;
            for u in BitMask(mask) {
                components[u] = cv;
            }
            forest.push((v, w));
            if min_edges <= forest.len() && forest.len() <= max_edges {
                let mut fc = forest.clone();
                fc.shrink_to_fit();
                output.push(fc);
            }
            i += 1;
        }
        output
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

    /// Output the graph in the graphviz dot format
    pub fn to_dot(&self) -> String {
        let mut s = "graph {\n".to_string();
        for &(v, w) in &self.edges {
            s.push_str(&format!("{v} -- {w}\n"));
        }
        s.push('}');
        s
    }
}

pub fn permute_edges(edges: &[(u8, u8)], perm: &[u8]) -> Vec<(u8, u8)> {
    edges
        .iter()
        .map(|&(u, v)| {
            let (a, b) = (perm[u as usize], perm[v as usize]);
            if a < b { (a, b) } else { (b, a) }
        })
        .collect()
}

pub fn permute_indexed_edges(edges: &[((u8, u8), u8)], perm: &[u8]) -> Vec<((u8, u8), u8)> {
    edges
        .iter()
        .map(|&((u, v), i)| {
            let (a, b) = (perm[u as usize], perm[v as usize]);
            if a < b { ((a, b), i) } else { ((b, a), i) }
        })
        .collect()
}

/// Remove the v-th bit of the mask, shifting the bits upper than v one place to the right.
pub fn delete_vertex_from_mask(v: u8, mask: u64) -> u64 {
    let m = (1 << v) - 1;
    (mask & m) | ((mask >> 1) & !m)
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

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// This is a really bad adding function, its purpose is to fail in this
// example.
#[allow(dead_code)]
fn bad_add(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    //#[test]
    fn test_bad_add() {
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(bad_add(1, 2), 3);
    }

    // Subforests function
    #[test]
    fn test_subforest() {
        let v: u8 = 8;

        let peterson_edges: Vec<Edge> = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (5, 8),
            (5, 9),
            (6, 7),
            (6, 9),
            (7, 8),
            (0, 5),
            (1, 6),
            (2, 8),
            (3, 9),
            (4, 7),
        ];
        let petersen: Graph = Graph::new(10, peterson_edges);

        let sfs: Vec<Vec<Edge>> = Vec::new();

        let t: usize = petersen.subforests(3, 3).len();
        let u: usize = 2730 / 6;

        assert_eq!(t, u);
    }

    //pub fn new(num_vertices: u8, mut edges: Vec<(u8, u8)>) -> Self {
}
