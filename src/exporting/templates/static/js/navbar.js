const MINWIDTH = 140;
const DEFAULT_NAV_WIDTH = 200;  // Default width of the navbar in pixels 
const IDNavWidth = "navwidth";  // ID in localstorage for the navigator width.
const SMALLWIDTHTHRES = 500;    // Threshold for small screen sizes.

let resizer = document.getElementById("navbar_resizer");
let navbar = document.querySelector("#navbar");
let navbarContainer = document.querySelector("#navbar-container");
let dragging = false;

//window.onload = initialize;
initialize();

function initialize() {
    let initWidth = loadStoredWidth();
    setWidth(initWidth);
};

function isClosed(navbar) {
    return navbar.style.visibility == "hidden";
}

function isSmallScreen() {
    return window.innerWidth <= SMALLWIDTHTHRES;
}

function loadStoredWidth() {
    // Load the width of the navbar from local storage. If it is not stored, then return the default. 
    let width = parseFloat(localStorage.getItem(IDNavWidth));
    if (isNaN(width)) {  // width had not been set 
        return DEFAULT_NAV_WIDTH;
    }
    return width
}


function setWidthRaw(width) {
    //Auxiliary function that sets the width of the nav
    let style = getComputedStyle(navbarContainer)
    let newWidth = (width - (parseFloat(style.paddingLeft) + parseFloat(style.paddingRight)));
    let maxwidth = parseFloat(getComputedStyle(document.body).getPropertyValue('--max-nav-width'));
    newWidth = Math.min(newWidth, maxwidth);
    navbarContainer.style.width = newWidth + "px";
}

function setWidth(width) {
    if (width < MINWIDTH && ~isClosed(navbar)) {
        closeNav();
    }
    else {
        if (isClosed(navbar)) { openNav(); }
        setWidthRaw(width);
    }
}

/* Toggle the sidebar open */
function onNavButton() {
    if (isSmallScreen()) {
        // On small screens, open full-width overlay
        navbarContainer.style.width = "100%";
        navbar.style.visibility = "visible";
        document.getElementById("navbutton").style.visibility = "hidden";
    } else {
        setWidth(DEFAULT_NAV_WIDTH);
    }
}

/* Open the sidebar */
function openNav() {
    animationExpand();
    navbar.style.visibility = "visible";
    document.getElementById("navbutton").style.visibility = "hidden";
    localStorage.setItem(IDNavWidth, navbar.style.width);
}

function animationExpand() {
    navbarContainer.classList.add('expand');
    setTimeout(() => {
        navbarContainer.classList.remove('expand')
    }, 500)
}

/* Set the width of the sidebar to 0 */
function closeNav() {
    // Disable the navbar itself 
    animationExpand();
    navbar.style.visibility = "hidden";
    // Enable the hamburger button 
    let hamburger = document.getElementById("navbutton");
    hamburger.style.visibility = "visible";

    // Collapse the navbar container
    navbarContainer.style.width = "0px";
    localStorage.setItem(IDNavWidth, 0);
}


if (resizer != null) {
    window.addEventListener("mousedown", mousedown);
    window.addEventListener("mouseup", mouseup);
    window.addEventListener("mousemove", mousemove);

    function mousedown(event) {
        if (resizer == event.target) {
            pauseEvent(event);
            dragging = true;
        }
    }

    function mousemove(event) {
        if (dragging) {
            pauseEvent(event);
            setWidth(event.clientX);
        }

    }

    function mouseup(event) {
        if (dragging) {
            localStorage.setItem(IDNavWidth, event.clientX);
        }
        dragging = false;
    }

}

function pauseEvent(e) {
    // Pause bubbling of an event to make sure selection is not activated during dragging. 
    if (e.stopPropagation) e.stopPropagation();
    if (e.preventDefault) e.preventDefault();
    e.cancelBubble = true;
    e.returnValue = false;
    return false;
}



