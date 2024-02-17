const selectElement = document.getElementById('filter_input');

function build_link_of_result(res) {
    return `<a href=${res.obj.path}>${fuzzysort.highlight(res)}</a>`
}

function present_search_results(results) {
    console.log(results[0].obj)
    math_html = results.map(match => `<li> ${build_link_of_result(match)}</li>`);
    document.getElementById("filter").innerHTML = math_html.join("\n");
}

async function getSearchIdx() { 
    let url = "/static/js/search_index.json";
    let response = await fetch(url); 
    return await response.json();
}

async function register_handler() { 
    let search_idx_raw = await getSearchIdx();
    selectElement.addEventListener(
        'input', event => {
            let search_query = event.target.value;
            if (search_query.length > 0) {
                let search_result = fuzzysort.go(
                    search_query, search_idx_raw, {
                        key: ['title'], limit: 50, threshold: -10000, all:false
                    }
                )
                present_search_results(search_result);
            }
        }
    );
}

register_handler();
