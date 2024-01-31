/* Set the width of the sidebar to 250px and the left margin of the page content to 250px */
function openNav() {
    document.getElementById("navbar").style.visibility = "visible";
    document.getElementById("navbutton").style.visibility = "hidden";
    document.getElementById("main").style.marginLeft = "340px"; 
  }
  
  /* Set the width of the sidebar to 0 and the left margin of the page content to 0 */
  function closeNav() {
    document.getElementById("navbar").style.visibility = "hidden";
    document.getElementById("navbutton").style.visibility = "visible";
    document.getElementById("main").style.marginLeft = "100px"; 
    // document.getElementById("main").style.marginLeft = "0";
  }
