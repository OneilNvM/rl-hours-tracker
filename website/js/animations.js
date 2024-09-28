function loader() {
    document.addEventListener("scroll", scrollToDivs);
}

function scrollToDivs() {
    let hoursDiv = document.getElementById('hours');
    let datesDiv = document.getElementById('dates');

    console.log("hours:", hoursDiv.offsetTop, "dates:", datesDiv.offsetTop)

    let pageBottom = window.innerHeight + document.documentElement.scrollTop;

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