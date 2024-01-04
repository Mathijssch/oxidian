//use super::errors::MathFindError;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}, cmp,
};

use log::{info,debug};

#[derive(Hash,Debug,PartialEq)]
pub struct Sanitization(pub String);

impl Sanitization {
    pub fn get_placeholder(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        return format!("{}", hasher.finish());
    }
}

enum MathState {
    ExpectOpen,
    ExpectClose,
}

struct DelimPair {
    pub open: String,
    pub close: String,
}

impl DelimPair {
    pub fn new(open: &str, close: &str) -> Self {
        DelimPair {
            open: String::from(open),
            close: String::from(close),
        }
    }
}

fn get_search_pattern<'a>(state: &MathState, pair: &'a DelimPair) -> &'a str {
    match state {
        MathState::ExpectOpen => &pair.open,
        MathState::ExpectClose => &pair.close,
    }
}

fn find_pairs(content: &str, delim: &DelimPair) -> Vec<Sanitization> {
    let mut result = vec![];
    let mut state = MathState::ExpectOpen;
    let mut curr_start = 0;
    let mut prev_open = None;

    //info!("Covering {:?}", &content.chars().take(2200).collect::<String>());

    while let Some(index) = content[curr_start..].find(get_search_pattern(&state, &delim)) { 
        let pattern = get_search_pattern(&state, delim);
        match state {
            MathState::ExpectOpen => {
                debug!("Found opening {} at index {} (local {})", pattern, curr_start + index, index);
                // Found an opening brace
                state = MathState::ExpectClose;
                prev_open = Some(curr_start + index);
            },
            MathState::ExpectClose => {
                debug!("Found closing {} at index {} (local {})", pattern, curr_start + index, index);
                // Found a closing brace
                if let Some(start_idx) = prev_open {
                    // Retrieve previous opening index.
                    let end_idx = cmp::min(
                        curr_start + index + pattern.len(),
                        content.len()
                    );
                    result.push(Sanitization(content[start_idx..end_idx].to_owned()));
                    state = MathState::ExpectOpen;
                    prev_open = None;
                }
            }
        };
        curr_start = cmp::min(curr_start + index + pattern.len(), 
                              content.len());
    };
    result
}

///Find code that is within pairs of given delimiters 
///(math and code) and replace them, by placeholders. 
///Otherwise, tags or other special symbols may be found in these places.
pub fn disambiguate_protected(content: &str) -> (String, Vec<Sanitization>) {
    let mut result = vec!();

    let pairs: [DelimPair; 6] = [
        DelimPair::new("$$", "$$"),
        DelimPair::new(r"\[", r"\]"),
        DelimPair::new("$", "$"),
        DelimPair::new(r"\(", r"\)"),
        DelimPair::new(r"```", r"```"),
        DelimPair::new(r"`", r"`")
    ];
    let mut new_string = String::from(content);
    for pair in pairs { 
        let sanitize = find_pairs(&new_string, &pair);
        for math_element in &sanitize { 
            new_string = new_string.replace(
                &math_element.0, &math_element.get_placeholder()
                );
        }
        result.extend(sanitize);
    }
    (new_string, result)
}


#[cfg(test)]
mod tests { 

    use std::assert_eq;

    use crate::oxidianlib::obs_placeholders::DelimPair;

    use super::{find_pairs, Sanitization};

    fn run_basic_test(query: &str, solution: Vec<&str>, open_delim: &str, close_delim: &str) {
        let sanitization: Vec<Sanitization> = solution
            .iter()
            .map(|s| Sanitization(s.to_string()))
            .collect();
        let delimiters = DelimPair::new(open_delim, close_delim);
        let pairs = find_pairs(query, &delimiters);
        assert_eq!(sanitization, pairs);
    }


    #[test]
    fn test_equation_at_the_end() {
        let content = "Something that contains an $equation$";
        run_basic_test(content, vec!["$equation$"], "$", "$");
    } 

    #[test]
    fn test_equation_internal() {
        let content = "Something that contains an $equation$ but does not end with it.";
        run_basic_test(content, vec!["$equation$"], "$", "$");
    } 

    #[test]
    fn test_equation_unicode() {
        let content = "Something that contains ümlauts and an $equation$ but does not end with it.";
        run_basic_test(content, vec!["$equation$"], "$", "$");
    } 

    #[test]
    fn test_equation_failure_case() {
        let content =
"
#literature #literature/misc\n# Configuration-Constrained Tube MPC\nBy [[M. Villanueva]] [[M. Müller]] [[B. Houska]] \n\n[]()\n\n[Try to open in Zotero](zotero://select/items/@villanueva_ConfigurationConstrainedTubeMPC_2022)\n\n<span class=\"abstract\">\nThis paper is about robust Model Predictive Control (MPC) for linear systems with additive and multiplicative uncertainty. A novel class of configurationconstrained polytopic robust forward invariant tubes is introduced, which admit a joint parameterization of their facets and vertices. They are the foundation for the development of novel Configuration-Constrained Tube MPC (CCTMPC) controllers that freely optimize the shape of their polytopic tube, subject to conic vertex configuration constraints, as well as associated vertex control laws by solving convex optimization problems online. It is shown that CCTMPC is—under appropriate assumptions—systematically less conservative than Rigid- and Homothetic- Tube MPC. Additionally, it is proven that there exist control systems for which CCTMPC is less conservative than Elastic Tube MPC, Disturbance Affine Feedback MPC, and Fully Parameterized Tube MPC.\n</span>\n\n## Notes\n\nInteresting new type of polyhedral tube MPC. Contains a nice theoretical treatment of polyhedral sets with a fixed shape matrix. \nThey claim that their tube-based MPC is superior to existing alternatives both in terms of computational complexity as conservatism, but this is proved in general. Furthermore, the class of tubes is quite novel and it is not very clear that it works well in practice, so I would not use these yet at the moment, as it would require extensive experimental testing of our own. \n\n### Template polyhedra \n\nSection 3 in the paper treats [[Template polyhedra]]. \n\n### Tube construction \n\nThe tubes constructed in this paper are of the form \n15532696941500219154\nwhere $Y$ is a fixed template matrix";
        run_basic_test(content, vec!["$Y$"], "$", "$");
    } 

    #[test]
    fn test_equation_failure_case_double() {
        let content =
            "#literature #literature/misc\n# Configuration-Constrained Tube MPC\nBy [[M. Villanueva]] [[M. Müller]] [[B. Houska]] \n\n[]()\n\n[Try to open in Zotero](zotero://select/items/@villanueva_ConfigurationConstrainedTubeMPC_2022)\n\n<span class=\"abstract\">\nThis paper is about robust Model Predictive Control (MPC) for linear systems with additive and multiplicative uncertainty. A novel class of configurationconstrained polytopic robust forward invariant tubes is introduced, which admit a joint parameterization of their facets and vertices. They are the foundation for the development of novel Configuration-Constrained Tube MPC (CCTMPC) controllers that freely optimize the shape of their polytopic tube, subject to conic vertex configuration constraints, as well as associated vertex control laws by solving convex optimization problems online. It is shown that CCTMPC is—under appropriate assumptions—systematically less conservative than Rigid- and Homothetic- Tube MPC. Additionally, it is proven that there exist control systems for which CCTMPC is less conservative than Elastic Tube MPC, Disturbance Affine Feedback MPC, and Fully Parameterized Tube MPC.\n</span>\n\n## Notes\n\nInteresting new type of polyhedral tube MPC. Contains a nice theoretical treatment of polyhedral sets with a fixed shape matrix. \nThey claim that their tube-based MPC is superior to existing alternatives both in terms of computational complexity as conservatism, but this is proved in general. Furthermore, the class of tubes is quite novel and it is not very clear that it works well in practice, so I would not use these yet at the moment, as it would require extensive experimental testing of our own. \n\n### Template polyhedra \n\nSection 3 in the paper treats [[Template polyhedra]]. \n\n### Tube construction \n\nThe tubes constructed in this paper are of the form \n15532696941500219154\nwhere $Y$ is a fixed template matrix, $\\mathcal{V}$ is the vertex set of \na given $\\beta$-contractive polytope $P(Y, \\sigma)$. \n\nand $\\mathbb{Y}_\\mathcal{V}$  is the vertex configuration domain of $P(Y,\\sigma)$:";
        run_basic_test(content, vec!["$Y$", "$\\mathcal{V}$", "$\\beta$", "$P(Y, \\sigma)$", "$\\mathbb{Y}_\\mathcal{V}$", "$P(Y,\\sigma)$"], "$", "$");
} 

    #[test]
    fn test_failure_case_display_math() {
        let content =
"
#literature #literature/misc\n# Configuration-Constrained Tube MPC\nBy [[M. Villanueva]] [[M. Müller]] [[B. Houska]] \n\n[]()\n\n[Try to open in Zotero](zotero://select/items/@villanueva_ConfigurationConstrainedTubeMPC_2022)\n\n<span class=\"abstract\">\nThis paper is about robust Model Predictive Control (MPC) for linear systems with additive and multiplicative uncertainty. A novel class of configurationconstrained polytopic robust forward invariant tubes is introduced, which admit a joint parameterization of their facets and vertices. They are the foundation for the development of novel Configuration-Constrained Tube MPC (CCTMPC) controllers that freely optimize the shape of their polytopic tube, subject to conic vertex configuration constraints, as well as associated vertex control laws by solving convex optimization problems online. It is shown that CCTMPC is—under appropriate assumptions—systematically less conservative than Rigid- and Homothetic- Tube MPC. Additionally, it is proven that there exist control systems for which CCTMPC is less conservative than Elastic Tube MPC, Disturbance Affine Feedback MPC, and Fully Parameterized Tube MPC.\n</span>\n\n## Notes\n\nInteresting new type of polyhedral tube MPC. Contains a nice theoretical treatment of polyhedral sets with a fixed shape matrix. \nThey claim that their tube-based MPC is superior to existing alternatives both in terms of computational complexity as conservatism, but this is proved in general. Furthermore, the class of tubes is quite novel and it is not very clear that it works well in practice, so I would not use these yet at the moment, as it would require extensive experimental testing of our own. \n\n### Template polyhedra \n\nSection 3 in the paper treats [[Template polyhedra]]. \n\n### Tube construction \n\nThe tubes constructed in this paper are of the form \n15532696941500219154\nwhere $Y$ is a fixed template matrix, $\\mathcal{V}$ is the vertex set of \na given $\\beta$-contractive polytope $P(Y, \\sigma)$. \n\nand $\\mathbb{Y}_\\mathcal{V}$  is the vertex configuration domain of $P(Y,\\sigma)$: \n$$\n    \\mathbb{Y}_{\\mathcal{V}} =  \\{ y \\in \\Re^m \\mid \\F_{\\mathcal{V}} (y) \\neq \\emptyset \\}, \n$$\nwhich can be explicitly expre
";
        run_basic_test(content, vec!["$$\n    \\mathbb{Y}_{\\mathcal{V}} =  \\{ y \\in \\Re^m \\mid \\F_{\\mathcal{V}} (y) \\neq \\emptyset \\}, \n$$"], "$$", "$$");
    } 
}

