const DARKMODE_KEY = "darkmode-desired";

let darkmodeDesired = localStorage.getItem(DARKMODE_KEY) == 'true'; 
////console.log("Darkmode requested: ", darkmodeDesired);
if (darkmodeDesired) {        
    ////console.log("dark mode");
    toggleLightMode();
}
////console.log("light mode");

function toggleLightMode() {
    let element = document.documentElement;
    element.classList.toggle("dark-mode");
    localStorage.setItem(DARKMODE_KEY, element.classList.contains("dark-mode"));
}
