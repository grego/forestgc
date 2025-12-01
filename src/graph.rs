use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
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
pub struct BigGraph {
    pub num_vertices: usize,
    pub neighbours: Vec<Vec<usize>>,
}

/// An iterator over the position of bits in a bitmask.
#[derive(Clone, Copy)]
pub struct BitPositions(pub u64);

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

/// Calculate the composition of two permutations, in natural order.
/// `i |-> b(a(i))`
pub fn compose(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().map(|&x| b[x as usize]).collect()
}

impl Graph {
    pub fn new(num_vertices: u8, edges: Vec<(u8, u8)>) -> Self {
        let mut edges: Vec<(u8, u8)> = edges.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
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
            for u in BitPositions(frontier) {
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

    /// Return the graph in the canonical form, an isomorphism into it and a list of
    /// non-trivial automorphisms of the canonical graph.
    #[inline(always)]
    pub fn canonical_label(&self) -> (Graph, Vec<u8>, Vec<Vec<u8>>) {
        let zero_colors = vec![0u128; self.num_vertices as usize];
        self.canonical_label_col(&zero_colors)
    }

    /// Return the graph in the canonical form, an isomorphism into it and a list of
    /// non-trivial automorphisms of the canonical graph,
    /// respecting the initial vertex coloring.
    #[inline(always)]
    pub fn canonical_label_col(&self, init_colors: &[u128]) -> (Graph, Vec<u8>, Vec<Vec<u8>>) {
        let (g, perms) = self.canonical_labels_col(init_colors);
        let base = perms[0].clone();
        let perms: Vec<_> = perms
            .into_iter()
            .skip(1)
            .map(|p| compose(&inverse(&base), &p))
            .collect();
        (g, base, perms)
    }

    /// Return the graph in the canonical form and a list of all isomorphisms into it.
    #[inline(always)]
    pub fn canonical_labels(&self) -> (Graph, Vec<Vec<u8>>) {
        let zero_colors = vec![0u128; self.num_vertices as usize];
        self.canonical_labels_col(&zero_colors)
    }

    /// Return the graph in the canonical form and a list of all isomorphisms into it,
    /// respecting the initial vertex coloring.
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

    /// Returns wheter a graph is isomorphic to the given graph
    pub fn is_isomorphic_to(&self, graph: &Graph) -> bool {
        let a: Graph = self.canonical_label().0;
        let b: Graph = graph.canonical_label().0;
        a.adj == b.adj
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
            for v in BitPositions(class_mask) {
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
                for v in BitPositions(class_mask) {
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

    /// Removes the given vertex
    pub fn remove_vertex(&self, v: u8) -> Graph {
        let num_vertices = self.num_vertices - 1;
        let dec = |w| if w < v { w } else { w - 1 };
        let edges = self
            .edges
            .iter()
            .filter(|&(u, w)| *u != v && *w != v)
            .map(|&(u, w)| (dec(u), dec(w)))
            .collect();
        let mut adj = self.adj.clone();
        adj.remove(v as usize);
        let mask = (1 << v) - 1;
        for a in adj.iter_mut() {
            *a = (*a & mask) | ((*a >> 1) & !mask);
        }
        Self {
            num_vertices,
            edges,
            adj,
        }
    }

    /// Returns whether a given vertex has valency 2
    pub fn has_valency2(&self, vertex: u8) -> bool {
        self.adj[vertex as usize].count_ones() == 2
    }

    /// Returns whether a given vertex has valency 3
    pub fn has_valency3(&self, vertex: u8) -> bool {
        self.adj[vertex as usize].count_ones() == 3
    }

    /// Returns whether a given vertex has valency at least `min` and at most `max`
    pub fn has_valency(&self, vertex: u8, min: u8, max: u8) -> bool {
        (self.adj[vertex as usize].count_ones() >= min as u32)
            && (self.adj[vertex as usize].count_ones() <= max as u32)
    }

    /// Return all valency 2 vertices
    pub fn vertices_valency2(&self) -> Vec<u8> {
        let mut edges: Vec<u8> = Vec::new();
        for i in 0..self.num_vertices {
            if self.has_valency2(i) {
                edges.push(i);
            }
        }
        edges
    }

    /// Return all valency 3 vertices
    pub fn vertices_valency3(&self) -> Vec<u8> {
        let mut edges: Vec<u8> = Vec::new();
        for i in 0..self.num_vertices {
            if self.has_valency3(i) {
                edges.push(i);
            }
        }
        edges
    }

    /// Return all vertices with valency at leas `min` and at most `max`
    pub fn vertices_valency(&self, min: u8, max: u8) -> Vec<u8> {
        let mut edges: Vec<u8> = Vec::new();
        for i in 0..self.num_vertices {
            if self.has_valency(i, min, max) {
                edges.push(i);
            }
        }
        edges
    }

    /// Returns whether the graph contains a loop, i.e. an edge from a vertex to itself
    /// Only works on graphs in the bipartite form
    pub fn contains_loop(&self) -> bool {
        !self.vertices_valency(1, 1).is_empty()
    }

    /// Returns whether a graph contains a pair of vertices with more then one edge between them
    /// Only works on graphs in the bipartite form
    pub fn is_multigraph(&self) -> bool {
        let mut seen = FxHashSet::default();
        for item in &self.adj {
            if !seen.insert(item) {
                return true;
            }
        }
        false
    }
    /// Returns the number of double edges of a multigraph
    /// Expect the graph to contain at most double edges, no triple edges or more
    pub fn count_double_edges(&self) -> u8 {
        let mut i: u8 = 0;
        let mut seen = FxHashSet::default();
        for item in &self.adj {
            if !seen.insert(item) {
                i += 1;
            }
        }
        i
    }

    // Returns the giths of a given graph, i.e. the size of the
    // smallest cycle in the graph or 0 if the graph is acyclic
    // It is expected the graph is in the bipartite format!
    pub fn girth(&self) -> u8 {
        if self.contains_loop() {
            return 1;
        }
        if self.is_multigraph() {
            return 2;
        }
        if self.num_vertices == (self.edges.len() + 1) as u8 {
            return 0;
        }

        let graph = self.simplify(true).0;

        let mut g: u8 = graph.num_vertices + 1;
        for v in 0..graph.num_vertices {
            let mut s: Vec<u8> = Vec::new();
            let mut r: VecDeque<u8> = VecDeque::new();
            r.push_back(v);
            let mut parent: Vec<u8> = vec![graph.num_vertices + 1; graph.num_vertices as usize];
            let mut d: Vec<u8> = vec![graph.num_vertices + 1; graph.num_vertices as usize];
            d[v as usize] = 0;

            while let Some(x) = r.pop_front() {
                s.push(x);
                for y in BitPositions(graph.adj[x as usize]) {
                    if y == (parent[x as usize] as usize) {
                        continue;
                    }
                    if !s.contains(&(y as u8)) {
                        parent[y] = x;
                        d[y] = d[x as usize] + 1;
                        r.push_back(y as u8);
                    } else {
                        g = g.min(d[x as usize] + d[y] + 1);
                    }
                }
            }
        }
        g
    }

    /// Contracts the given edge, expects to be called on simple graphs!
    /// Crashed if the edge is not present in the graph!
    pub fn contract_edge(&self, edge: Edge) -> Graph {
        assert!(
            self.edges.contains(&edge),
            "Graph does not contain the edge"
        );

        let (u, w) = (edge.0.min(edge.1), edge.0.max(edge.1));
        let f = |a| if a == w { u } else { a - (a > w) as u8 };
        let edges = self
            .edges
            .iter()
            .filter(|&&e| e != (u, w) && e != (w, u))
            .map(|&(a, b)| (f(a), f(b)))
            .collect();

        Graph::new(self.num_vertices - 1, edges)
    }

    /// Contract the neigborhood of a vertex.
    /// Returns the contracted graph, along with the map from old vertices to new ones.
    pub fn contract_neighborhood(&self, v: u8) -> (Self, Vec<u8>) {
        let mut morphism: Vec<_> = (0..self.num_vertices).collect();
        let neigh = self.adj[v as usize] | (1 << v);
        let size = neigh.count_ones() as u8;

        // Make sure to not create a double edge.
        let mut edges = self.edges.clone();
        for (w, &a) in self.adj.iter().enumerate() {
            for u in BitPositions(neigh & a).skip(1) {
                if let Some(i) = edges
                    .iter()
                    .position(|e| *e == (u.min(w) as u8, u.max(w) as u8))
                {
                    edges.swap_remove(i);
                }
            }
        }

        let min = neigh.trailing_zeros() as u8;
        for u in BitPositions(neigh).skip(1) {
            morphism[u] = min;
            for m in morphism.iter_mut().skip(u + 1) {
                *m -= 1;
            }
        }
        let f = |a| Some(morphism[a as usize]).filter(|_| a != v);
        let edges = edges
            .iter()
            .filter_map(|&(a, b)| Some((f(a)?, f(b)?)))
            .collect();
        (Graph::new(self.num_vertices - size + 1, edges), morphism)
    }

    /// Contracts the neighborhoods of the given vertices
    pub fn contract_multiple_neighborhoods(&self, vertices: &[u8]) -> Graph {
        let mut vert = vertices.to_vec();
        let mut g = self.clone();

        while let Some(v) = vert.pop() {
            let (gr, map) = g.contract_neighborhood(v);
            vert = compose(&vert, &map);
            g = gr;
        }
        g
    }

    /// Turn valency 2 vertices into new edges.
    /// Return the list of original indices of the new edges.
    /// If retain_multiedges is true, it returns a graph with only a single edge in place of a multiedge
    /// if retain_multiedges is false, it returns a graph with no edge in place of a multiedge
    pub fn simplify(&self, retain_multiedges: bool) -> (Self, Vec<(Edge, u8)>) {
        let mut new_edges = FxHashMap::default();
        let mut perm: Vec<_> = (0..self.num_vertices).collect();
        let mut new_v = 0;
        for (i, &a) in self.adj.iter().enumerate() {
            match a.count_ones() {
                2 => {
                    let mut iter = BitPositions(a);
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

    /// Returns the given graph in the form of a bipartite graph,
    /// where the second color vertices represent edges
    pub fn to_bipartite(&self) -> Graph {
        let mut i: u8 = 0;
        let mut edges = Vec::new();
        for (v1, v2) in self.edges.iter() {
            let e1: Edge = (*v1, self.num_vertices + i);
            edges.push(e1);
            if v2 != v1 {
                let e2: Edge = (*v2, self.num_vertices + i);
                edges.push(e2);
            }
            i += 1;
        }
        Graph::new(self.num_vertices + i, edges)
    }

    /// Find all subforests with the given minimum and maximum size.
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
                for u in BitPositions(mask) {
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
            for u in BitPositions(mask) {
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

    /// Convert the graph to a multigraph, with new edges given by degree 2 vertices.
    pub fn to_multigraph(&self) -> Self {
        let mut new_edges = Vec::new();
        let mut perm: Vec<_> = (0..self.num_vertices).collect();
        let mut new_v = 0;
        for (i, &a) in self.adj.iter().enumerate() {
            match a.count_ones() {
                1 => {
                    let v = a.trailing_zeros();
                    new_edges.push((v as u8, v as u8));
                }
                2 => {
                    let mut iter = BitPositions(a);
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

    /// Compute the number of connected components of a graph, along with an assignment
    /// of connected components to vertices.
    pub fn connected_components(&self) -> (u8, Vec<u8>) {
        let mut components = vec![0; self.num_vertices as usize];
        let mut stack = Vec::new();
        let mut i: u8 = 0;
        while let Some(u) = components.iter().position(|&e| e == 0) {
            i += 1;
            components[u] = i;
            stack.push(u);
            while let Some(v) = stack.pop() {
                for w in BitPositions(self.adj[v]) {
                    if components[w] == 0 {
                        components[w] = i;
                        stack.push(w);
                    }
                }
            }
        }
        (i, components)
    }

    /// Returns whether a graph is 3-edge connected
    /// Expected to be called on simple 3 valent graph only and in the bipartite form!
    pub fn is_3edge_connected(&self) -> bool {
        let edges = self.vertices_valency2();

        for e1 in edges.iter() {
            for e2 in edges.iter() {
                if e1 <= e2 {
                    continue;
                }
                let g = self.remove_vertex(*e1).remove_vertex(*e2);
                if g.connected_components().0 > 1 {
                    return false;
                }
            }
        }
        true
    }

    /// Returns whether a graph is k-edge connected
    /// Expected to be called on simple 3 valent graph only and in the bipartite form!
    /// If `proper` is set to `true`, each component after deleting `k-1` edges
    /// must have more than one vertex.
    pub fn is_k_edge_connected(&self, k: usize, proper: bool) -> bool {
        if k == 1 {
            return true;
        }
        if !self.is_k_edge_connected(k - 1, proper) {
            return false;
        }

        let edges = self.vertices_valency2();
        let ktuples = get_ktuples(&edges, k - 1);

        'outer: for kt in ktuples {
            let mut g = self.clone();
            for &v in kt.iter().rev() {
                g = g.remove_vertex(v);
            }
            let (n, comps) = g.connected_components();
            if n > 1 {
                if proper {
                    for i in 1..=n {
                        let count = comps.iter().filter(|&&c| c == i).count();
                        if count == 1 {
                            continue 'outer;
                        }
                    }
                }
                return false;
            }
        }

        true
    }

    /// Returns whether a graph is 3-edge connected
    /// Expected to be called on simple 3 valent graph only and in the bipartite form!
    pub fn is_3vertex_connected(&self) -> bool {
        let vertices = self.vertices_valency(3, 255);

        for e1 in vertices.iter() {
            for e2 in vertices.iter() {
                if e1 <= e2 {
                    continue;
                }
                let g = self.remove_vertex(*e1).remove_vertex(*e2);
                if g.connected_components().0 > 1 {
                    return false;
                }
            }
        }
        true
    }
}

impl BigGraph {
    pub fn new(num_vertices: usize, edges: Vec<(usize, usize)>) -> Self {
        let mut edges: Vec<_> = edges.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
        edges.sort_unstable();

        let mut neighbours = vec![Vec::new(); num_vertices];

        for &(u, v) in edges.iter() {
            neighbours[u].push(v);
            neighbours[v].push(u);
        }

        BigGraph {
            num_vertices,
            neighbours,
        }
    }

    /// Compute the number of connected components of a graph, along with an assignment
    /// of connected components to vertices.
    pub fn connected_components(&self) -> (usize, Vec<usize>) {
        let mut components = vec![0; self.num_vertices];
        let mut stack = Vec::new();
        let mut i = 0;
        while let Some(u) = components.iter().position(|&e| e == 0) {
            i += 1;
            components[u] = i;
            stack.push(u);
            while let Some(v) = stack.pop() {
                for &w in &self.neighbours[v] {
                    if components[w] == 0 {
                        components[w] = i;
                        stack.push(w);
                    }
                }
            }
        }
        (i, components)
    }

    /// Is the big graph vertex connected?
    pub fn is_vertex_connected(&mut self) -> Option<(usize, usize, Vec<usize>)> {
        for i in 0..self.num_vertices {
            let neigh = mem::take(&mut self.neighbours[i]);
            let (j, c) = self.connected_components();
            if j > 2 {
                return Some((i, j, c));
            }
            self.neighbours[i] = neigh;
        }
        None
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

/// Permute the bits of the mask using the provided permutation.
pub fn permute_mask(mask: u64, perm: &[u8]) -> u64 {
    let mut m = 0;
    for i in BitPositions(mask).map(|j| perm[j]) {
        m |= 1 << i;
    }
    m
}

/// Get all ktuples of the given array.
pub fn get_ktuples<T: Copy>(arr: &[T], k: usize) -> Vec<Vec<T>> {
    let mut ktuples = Vec::new();
    get_ktuples_iter(k, &mut ktuples, Vec::new(), arr);
    ktuples
}

fn get_ktuples_iter<T: Copy>(k: usize, ktuples: &mut Vec<Vec<T>>, head: Vec<T>, rest: &[T]) {
    if k == 0 {
        return;
    }

    if k == 1 {
        for &i in rest {
            let mut v = head.clone();
            v.push(i);
            ktuples.push(v);
        }
        return;
    }

    for (j, &i) in rest.iter().enumerate() {
        let mut v = head.clone();
        v.push(i);
        get_ktuples_iter(k - 1, ktuples, v, &rest[j + 1..]);
    }
}

impl Iterator for BitPositions {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let BitPositions(mask) = *self;
        if mask == 0 {
            return None;
        }
        let u = mask.trailing_zeros() as usize;
        *self = BitPositions(mask & (mask - 1));
        Some(u)
    }
}
