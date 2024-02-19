// "Disable" link by removing the href property
links = document.querySelectorAll('a.broken');
links.forEach((link) => {
    link.href = '';
    link.style.textDecoration = "none";
});
