use crate::graph::{BitPositions, Graph, ordered_ktuples};
use crate::graph::{compose, inverse, permute_mask, sign_subset};

use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::{Display, Formatter, Write};

/// Graph with all subforests. It is a multigraph without tadpoles, with edges of
/// valency at least 3.
/// Valency 2 vertices represent the edges.
pub struct ForestedGraph {
    /// The underlying graph.
    graph: Graph,
    /// All automorphisms of the graph.
    perms: Vec<Vec<u8>>,
    /// Bitmask of the valency 2 vertices representing the edges.
    edges: u64,
    /// Masks of all the suboferst of the graph, up to isomorphism, that don't have an odd
    /// automorphism.
    /// Every isomorphism class has lexicographically the smallest representative.
    subforests: Vec<u64>,
    /// Does the forested graph use the odd sign convention?
    odd: bool,
    /// The marked hairs on the graph.
    hairs: Vec<u8>,
}

/// The result of the unmarking differential on all subforests in a single graph.
pub struct UnmarkDifferential {
    /// All subforests obtained by unmarking an edge.
    smaller_forests: Vec<u64>,
    /// The columns of the differential - `(subforest, value)`.
    columns: Vec<Vec<(u64, i8)>>,
}

/// The result of the contracting differential on all subforests in a single graph.
pub struct ContractDifferential {
    /// G6 value of all contracted graphs, along with all subforests.
    contracted_graphs: Vec<(String, Vec<u64>)>,
    /// The columns of the differential - `((graph_index, subforest), value)`.
    columns: Vec<Vec<((usize, u64), i8)>>,
}

/// A table, providing for each graph and its subforest the index of its row.
pub struct GraphTable {
    /// Different underlying graphs.
    graphs: FxHashMap<String, usize>,
    /// All subforests in the graphs.
    forests: Vec<Vec<u64>>,
    /// Pre-computed indices for the rows.
    indices: Vec<usize>,
}

impl ForestedGraph {
    /// Find all subforests, up to isomorphism, of the graph, with the given size.
    /// Edges on multiedges can be disabled.
    pub fn new(g: &Graph, forest_size: usize, forests_on_multiedges: bool, odd: bool) -> Self {
        Self::in_range(g, forest_size, forest_size, forests_on_multiedges, odd)
    }

    /// Find all subforests, up to isomorphism, of the graph, in the specified range.
    /// Edges on multiedges can be disabled.
    pub fn in_range(
        g: &Graph,
        min: usize,
        max: usize,
        forests_on_multiedges: bool,
        odd: bool,
    ) -> Self {
        let (graph, _, perms) = g.canonical_label();

        Self::with_autos(
            graph,
            perms,
            min,
            max,
            forests_on_multiedges,
            odd,
            Vec::with_capacity(0),
        )
    }

    /// Find all subforests, up to isomorphism, of the graph, with all possible choice of hairs,
    /// in the specified range.
    /// Edges on multiedges can be disabled.
    pub fn hairy(
        g: &Graph,
        min: usize,
        max: usize,
        forests_on_multiedges: bool,
        odd: bool,
        hairs: u8,
    ) -> Vec<Self> {
        let (graph, _, perms) = g.canonical_label();

        let mut graphs = Vec::new();

        if hairs == 0 {
            return vec![Self::with_autos(
                graph,
                perms,
                min,
                max,
                forests_on_multiedges,
                odd,
                Vec::with_capacity(0),
            )];
        }

        let unary: Vec<_> = (0..graph.num_vertices)
            .filter(|&v| graph.adj[v as usize].count_ones() == 1)
            .collect();
        for h in ordered_ktuples(&unary, hairs as usize) {
            if let Some(p) = hair_preserving_perms(&perms, &h) {
                let g = Self::with_autos(graph.clone(), p, min, max, forests_on_multiedges, odd, h);
                graphs.push(g);
            }
        }
        graphs
    }

    fn with_autos(
        graph: Graph,
        mut perms: Vec<Vec<u8>>,
        min: usize,
        max: usize,
        forests_on_multiedges: bool,
        odd: bool,
        hairs: Vec<u8>,
    ) -> Self {
        let everything = (1 << graph.num_vertices) - 1;

        let signs: Vec<_> = if !odd {
            vec![1; perms.len()]
        } else {
            perms
                .iter()
                .map(|p| sign_halfedges(p, &graph.adj))
                .collect()
        };

        let (gs, dict) = graph.simplify(forests_on_multiedges);
        let subfs = gs.subforests(min, max);
        let mut subforests = FxHashSet::default();
        for subf in subfs {
            let mut mask = 0_u64;
            for i in subf
                .iter()
                .filter_map(|e| dict.binary_search_by_key(e, |&(r, _)| r).ok())
            {
                mask |= 1 << dict[i].1;
            }

            if odd {
                mask = everything & !mask;
            }
            if let Some((mut csf, _)) = canonical_subforest_even(mask, &perms, &signs) {
                if odd {
                    csf = everything & !csf;
                }
                subforests.insert(csf);
            }
        }
        let mut subforests: Vec<_> = subforests.into_iter().collect();
        subforests.sort_unstable();

        let mut edges = 0;
        for &(_, i) in &dict {
            edges |= 1 << i;
        }

        perms.shrink_to_fit();
        for perm in &mut perms {
            perm.shrink_to_fit();
        }
        subforests.shrink_to_fit();

        Self {
            graph,
            perms,
            edges,
            subforests,
            odd,
            hairs,
        }
    }
    /// Find all subforests, up to isomorphism, of the graph.
    pub fn all(g: &Graph, odd: bool, hairs: u8) -> Vec<Self> {
        Self::hairy(g, 0, g.num_vertices as usize - 1, true, odd, hairs)
    }

    /// Create a new forested graph with only the provided subforests.
    pub fn with_subforests(g: &Graph, forests: &[Vec<u8>], odd: bool) -> Self {
        let graph = g.clone();
        let perms = g.automorphisms();
        let mut subforests = Vec::new();
        let mut edges = 0;
        for f in forests {
            let mut mask = 0;
            for i in f {
                assert_eq!(graph.adj[*i as usize].count_ones(), 2);
                mask |= 1 << i;
            }
            subforests.push(mask);
            edges |= mask;
        }
        Self {
            graph,
            perms,
            edges,
            subforests,
            odd,
            hairs: Vec::with_capacity(0),
        }
    }

    /// Compute the unmarking differential.
    pub fn d_unmark(&self) -> UnmarkDifferential {
        let mut smaller_forests = FxHashSet::default();
        let mut columns = Vec::new();
        let everything = (1 << self.graph.num_vertices) - 1;
        let signs: Vec<_> = if !self.odd {
            vec![1; self.perms.len()]
        } else {
            self.perms
                .iter()
                .map(|p| sign_halfedges(p, &self.graph.adj))
                .collect()
        };

        for &mask in &self.subforests {
            let mut col = Vec::new();
            let mut sign = 1;
            // let compl = !mask & self.edges;
            for i in BitPositions(mask) {
                let mut m = mask & !(1 << i);
                if self.odd {
                    m = everything & !m;
                    sign = (-1_i8).pow((((1 << i) - 1) & m).count_ones());
                }
                if let Some((mut f, s)) = canonical_subforest(m, &self.perms, &signs) {
                    if self.odd {
                        f = everything & !f;
                    }
                    smaller_forests.insert(f);
                    add_or_push(&mut col, f, s * sign);
                }
                sign *= -1;
            }
            columns.push(col);
        }
        let mut smaller_forests: Vec<_> = smaller_forests.into_iter().collect();
        smaller_forests.sort_unstable();

        smaller_forests.shrink_to_fit();
        columns.shrink_to_fit();
        for col in &mut columns {
            col.shrink_to_fit();
        }

        UnmarkDifferential {
            smaller_forests,
            columns,
        }
    }

    /// Compute the contracting differential.
    pub fn d_contract(&self) -> ContractDifferential {
        let mut contracted_graphs: Vec<(String, Vec<Vec<u8>>)> = Vec::new();
        let mut contracted_indices = vec![(0, Vec::new()); self.graph.num_vertices as usize];
        let mut signs = Vec::new();
        let everything = (1 << (self.graph.num_vertices)) - 1;
        let smaller = (1 << (self.graph.num_vertices - 2)) - 1;
        let mut vertices: u64 = 0;
        if self.odd {
            for (i, m) in self.graph.adj.iter().enumerate() {
                if m.count_ones() >= 3 {
                    vertices |= 1 << i;
                }
            }
        }

        let mut oddsigns = vec![1; self.graph.num_vertices as usize];
        let mut odddeleted = vec![0; self.graph.num_vertices as usize];
        for i in BitPositions(self.edges) {
            let (g, m) = self.graph.contract_neighborhood(i as u8);
            let (g, base, mut perms) = g.canonical_label();
            let mut to_canon = compose(&m, &base);
            let mut g6 = g.to_g6();

            if !self.hairs.is_empty() {
                let mut hairs = compose(&self.hairs, &to_canon);
                let (c, ps) = canonical_hairs(&perms, &mut hairs);
                if let Some(p) = c {
                    to_canon = compose(&to_canon, &p);
                }
                perms = ps;

                for &hair in &hairs {
                    g6.push_str(&format!("-{:X}", hair));
                }
            }

            if self.odd {
                let (v, _) = to_canon
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|&(j, v)| j != i && *v == to_canon[i])
                    .unwrap();
                oddsigns[i] = sign_halfedges(&to_canon, &self.graph.adj);
                odddeleted[i] = 1 << v;
            }

            if let Some(k) = contracted_graphs.iter().position(|(g, _)| g == &g6) {
                contracted_indices[i] = (k, to_canon);
                continue;
            }
            let s = if !self.odd {
                vec![1; perms.len()]
            } else {
                perms.iter().map(|p| sign_halfedges(p, &g.adj)).collect()
            };
            contracted_indices[i] = (contracted_graphs.len(), to_canon);
            contracted_graphs.push((g6, perms));
            signs.push(s);
        }

        let mut contracted_forests = vec![FxHashSet::default(); self.graph.num_vertices as usize];
        let mut columns = Vec::new();
        for &mask in &self.subforests {
            let mut col = Vec::new();
            let mut sign = 1;
            for i in BitPositions(mask) {
                let &(j, ref to_canon) = &contracted_indices[i];
                // must be a marked edge on a double edge with the odd sign convention
                // whose contraction produces a loop, i.e. 0 in this convention
                if to_canon.is_empty() {
                    continue;
                }
                let (_, perms) = &contracted_graphs[j];
                let ma = if !self.odd { mask } else { everything & !mask };
                let ss = sign_subset(to_canon, BitPositions(ma & !(1 << i) & !odddeleted[i]));
                let mut m = permute_mask(mask & !(1 << i), to_canon);
                if self.odd {
                    m = smaller & !m;
                    let os = (-1_i8).pow(((odddeleted[i] - 1) & ma).count_ones());
                    sign = os * oddsigns[i];
                }
                if let Some((mut f, s)) = canonical_subforest(m, perms, &signs[j]) {
                    if self.odd {
                        f = everything & !f;
                    }
                    contracted_forests[j].insert(f);
                    add_or_push(&mut col, (j, f), s * sign * ss);
                }
                sign *= -1;
            }
            columns.push(col);
        }
        let mut contracted_graphs: Vec<_> = contracted_forests
            .drain(..)
            .map(|mut cf| cf.drain().collect::<Vec<u64>>())
            .zip(contracted_graphs.drain(..))
            .map(|(fs, (g, _))| (g, fs))
            .collect();
        contracted_graphs.shrink_to_fit();
        for (_, v) in &mut contracted_graphs {
            v.shrink_to_fit();
        }
        for col in &mut columns {
            col.shrink_to_fit();
        }

        ContractDifferential {
            contracted_graphs,
            columns,
        }
    }

    pub fn subforests(&self) -> &[u64] {
        &self.subforests
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Keep only the subforests that satisfy the given predicate.
    pub fn filter<F: Fn(u64) -> bool>(&self, f: F) -> Self {
        let subforests = self.subforests.iter().copied().filter(|u| f(*u)).collect();
        Self {
            subforests,
            graph: self.graph.clone(),
            perms: self.perms.clone(),
            edges: self.edges,
            odd: self.odd,
            hairs: self.hairs.clone(),
        }
    }

    pub fn graph_string(&self) -> String {
        let mut f = String::new();
        write!(f, "{}", self.graph).unwrap();
        for &h in &self.hairs {
            write!(f, "-{h:X}").unwrap();
        }
        f
    }
}

impl UnmarkDifferential {
    /// Return the list of subforests obtained by unmarking an edge.
    pub fn smaller_forests(&self) -> &[u64] {
        &self.smaller_forests
    }

    /// Return the list of signed unmarkings for each edge.
    pub fn columns(&self) -> &[Vec<(u64, i8)>] {
        &self.columns
    }

    /// List the sparse matrix entries for this differential, with an optional shift
    /// of rows and columns.
    pub fn matrix_entries(
        &self,
        col_shift: u32,
        row_shift: u32,
    ) -> impl Iterator<Item = (u32, u32, i8)> {
        self.columns.iter().enumerate().flat_map(move |(i, c)| {
            c.iter().flat_map(move |(m, s)| {
                let j = self.smaller_forests.binary_search(m).ok()?;
                Some((j as u32 + row_shift, i as u32 + col_shift, *s))
            })
        })
    }

    /// Filter the smaller forests by the provided predicate.
    pub fn filter<F: Fn(u64) -> bool>(self, f: F) -> Self {
        let smaller_forests = self.smaller_forests.into_iter().filter(|u| f(*u)).collect();
        Self {
            smaller_forests,
            columns: self.columns,
        }
    }
}

impl ContractDifferential {
    pub fn contracted_graphs(&self) -> &[(String, Vec<u64>)] {
        &self.contracted_graphs
    }

    pub fn columns(&self) -> &[Vec<((usize, u64), i8)>] {
        &self.columns
    }
}

impl GraphTable {
    /// Create a new graphtable from an iterator of a graph G6 string and a list of subforests.
    pub fn new<'a>(iter: impl Iterator<Item = (String, &'a [u64])>) -> Self {
        let mut num = 0;
        let mut graphs = FxHashMap::default();
        let mut forest_sets = Vec::new();
        for (g, fs) in iter {
            let j = graphs.entry(g).or_insert_with(|| {
                forest_sets.push(FxHashSet::default());
                num += 1;
                num - 1
            });
            for &i in fs {
                forest_sets[*j].insert(i);
            }
        }
        let forests: Vec<_> = forest_sets
            .drain(..)
            .map(|mut fs| {
                let mut fs: Vec<_> = fs.drain().collect();
                fs.sort_unstable();
                fs
            })
            .collect();

        let mut indices = vec![0; num + 1];
        let mut sum = 0;
        for i in 0..num {
            sum += forests[i].len();
            indices[i + 1] = sum;
        }
        Self {
            graphs,
            forests,
            indices,
        }
    }

    /// Make a graph indexing table from the provided list of forested graphs.
    pub fn from_forested_graphs(fgs: Vec<ForestedGraph>) -> Self {
        let (mut graphs, mut forests, mut indices) = (
            FxHashMap::default(),
            Vec::with_capacity(fgs.len()),
            Vec::with_capacity(fgs.len() + 1),
        );
        indices.push(0);
        let mut sum = 0;
        for (i, fg) in fgs.into_iter().enumerate() {
            graphs.insert(fg.graph.to_g6(), i);
            sum += fg.subforests.len();
            indices.push(sum);
            forests.push(fg.subforests);
        }
        Self {
            graphs,
            forests,
            indices,
        }
    }

    /// Get the number of the forested graphs in the table.
    pub fn size(&self) -> usize {
        *self.indices.last().unwrap()
    }

    /// Get the number of the forested graphs in the table.
    pub fn num_graphs(&self) -> usize {
        self.forests.len()
    }

    pub fn get_index(&self, g: &str, forest: u64) -> Option<usize> {
        let &index = &self.graphs.get(g)?;
        Some(self.forests[*index].binary_search(&forest).ok()? + self.indices[*index])
    }
}

/// Compute the canonical form of a subforest, given a list of graph automorphisms.
/// If it has an odd automorphism, return None.
/// Otherwise, return a touple `(forest, sign)` where `forest` is its representing
/// class and `sign` its sign.
#[inline]
pub fn canonical_subforest(mask: u64, perms: &[Vec<u8>], signs: &[i8]) -> Option<(u64, i8)> {
    let mut cm = mask;
    let mut sign = 1;
    let mut _autos = 1;
    for (i, perm) in perms.iter().enumerate() {
        let m = permute_mask(mask, perm);
        if m == mask {
            if signs[i] * sign_subset(perm, BitPositions(mask)) == -1 {
                return None;
            }
            _autos += 1;
        } else if m < cm {
            cm = m;
            sign = signs[i] * sign_subset(perm, BitPositions(mask));
        }
    }
    Some((cm, sign))
}

/// Compute the canonical form of a subforest, given a list of graph automorphisms.
/// If it has an odd automorphism, return None.
/// Otherwise, return a touple `(forest, sign)` where `forest` is its representing
/// class and `sign` its sign.
#[inline]
pub fn canonical_subforest_even(mask: u64, perms: &[Vec<u8>], signs: &[i8]) -> Option<(u64, i8)> {
    let mut cm = mask;
    let mut sign = 1;
    let mut _autos = 1;
    for (i, perm) in perms.iter().enumerate() {
        let m = permute_mask(mask, perm);
        let s = signs.get(i).unwrap_or(&1) * sign_subset(perm, BitPositions(mask));
        if m == mask {
            if s == -1 {
                return None;
            }
            _autos += 1;
        } else if m < cm {
            cm = m;
            sign = s;
        }
    }
    Some((cm, sign))
}

fn sign_halfedges(perm: &[u8], adj: &[u64]) -> i8 {
    let mut s = 1;
    for &a in adj.iter().filter(|a| a.count_ones() == 2) {
        let v = a.trailing_zeros();
        let w = (a & !(1 << v)).trailing_zeros();
        if perm[v as usize] > perm[w as usize] {
            s *= -1;
        }
    }
    s
}

/// Add the value to the `(key, value)` list,
/// or push a new one if not already present.
#[inline]
fn add_or_push<T: Eq>(l: &mut Vec<(T, i8)>, k: T, v: i8) {
    for i in 0..l.len() {
        if l[i].0 == k {
            l[i].1 += v;
            if l[i].1 == 0 {
                l.swap_remove(i);
            }
            return;
        }
    }
    l.push((k, v));
}

fn hair_preserving_perms(perms: &[Vec<u8>], hairs: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut p = Vec::new();
    for perm in perms {
        let ph = compose(hairs, perm);
        if ph.as_slice() < hairs {
            return None;
        }
        if ph == hairs {
            p.push(perm.clone());
        }
    }
    Some(p)
}

fn canonical_hairs(perms: &[Vec<u8>], hairs: &mut Vec<u8>) -> (Option<Vec<u8>>, Vec<Vec<u8>>) {
    let mut p = Vec::new();
    let mut canperm = None;
    let orig_hairs = hairs.clone();
    for perm in perms {
        let ph = compose(&orig_hairs, perm);
        if ph.as_slice() < hairs {
            canperm = Some(perm.clone());
            *hairs = ph;
            p.drain(..);
        } else if ph.as_slice() == hairs.as_slice() {
            let mut perm = perm.clone();
            if let Some(ref q) = canperm {
                perm = compose(&inverse(q), &perm);
            }
            p.push(perm.clone());
        }
    }
    (canperm, p)
}

impl Display for ForestedGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.graph)?;
        for &h in &self.hairs {
            write!(f, "-{h:X}")?;
        }
        for &m in self.subforests() {
            write!(f, " {m:X}")?;
        }
        Ok(())
    }
}

impl Display for UnmarkDifferential {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &m in self.smaller_forests() {
            write!(f, " {m:X}")?;
        }
        Ok(())
    }
}

impl Display for GraphTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, fs) in self.forests.iter().enumerate() {
            if let Some((g, _)) = self.graphs.iter().find(|(_, j)| **j == i) {
                write!(f, "{g}")?;
            }
            for &m in fs {
                write!(f, " {m:X}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
