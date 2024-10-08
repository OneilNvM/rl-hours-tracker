function loader() {
    document.addEventListener("scroll", scrollToDivs);
}

// *Future Implementation*

// function navBarAnimations() {
//     const header = document.getElementById('header');
//     const nav = document.getElementById('nav');

//     if (header.offsetHeight < document.documentElement.scrollTop) {
//         nav.style.display = "flex"
//         nav.style.animationName = "drop-down-nav"
//     } else {
//         nav.style.animationName = "reverse-drop-down-nav"
//         setTimeout(() => nav.style.display = "none", 450);
//     }
// }

function scrollToDivs() {
    const hoursDiv = document.getElementById('hours');
    const datesDiv = document.getElementById('dates');
    const header = document.getElementById('header');
    const nav = document.getElementById('nav');

    let pageBottom = window.innerHeight + document.documentElement.scrollTop;

    if (header.offsetHeight < document.documentElement.scrollTop) {
        nav.style.display = "flex"
    } else {
        nav.style.display = "none"
    }

    if (hoursDiv.offsetTop < pageBottom) {
        hoursDiv.classList.add("visible");
    } else {
        hoursDiv.classList.remove("visible");
    }

    if (datesDiv.offsetTop < pageBottom) {
        datesDiv.classList.add("visible");
    } else {
        datesDiv.classList.remove("visible");
    }

}

document.addEventListener("DOMContentLoaded", loader);