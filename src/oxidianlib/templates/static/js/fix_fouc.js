// Helper function
let domReady = (cb) => { 
    (document.readyState === 'interactive' || document.readyState === 'complete')
    ? cb()
    : document.addEventListener('DOMContentLoaded', cb);
};

domReady(() => {
  // Display body when DOM is loaded
  console.log("Loaded! Show it!")
  document.body.style.visibility = 'visible';
});
