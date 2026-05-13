document.querySelectorAll('figure').forEach(
  (fig, i) => {
    const caption = fig.querySelector('figcaption');
    const alt = fig.alt || 'Figure';
    caption.innerHTML = `<b>Figure ${i + 1}</b>. ${caption.innerHTML}`;
    fig.dataset.creflabel = `Figure ${i+1}`;
      fig.dataset.reflabel = `${i+1}`
  }
)


function FindTarget(s) {
    s.trim();
    console.log("Cref label: ", s);
    const target = document.querySelector(s);
    console.log("Found target: ", target);
    return target;
}


function handleTarget(a) {
    console.log("Found cref:", a);
    const refId = a.innerHTML;
    const refs = refId.split(',').map(r => r.trim());
    const targets = refs.map(FindTarget);

    if (targets.length === 1) {
        const target = targets[0];
        console.log("Binding ", refId, " to ", target.dataset.creflabel);
        a.innerHTML = `<a href="#${refId}">${target.dataset.creflabel}</a>`;
    } else if (targets.length > 1) {
        console.log("Binding ", refId, " to multiple targets");
        const links = refs.map((r, i) => {
            const t = targets[i];
            return t ? `<a href="#${r}">${t.dataset.reflabel}</a>` : r;
        });

        const formatted = links.length > 2
            ? links.slice(0, -1).join(', ') + ' and ' + links[links.length - 1]
            : links.join(' and ');

        a.innerHTML = `Figures ${formatted}`;
    }
}

document.querySelectorAll('cref').forEach(handleTarget);
