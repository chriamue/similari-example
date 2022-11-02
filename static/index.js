var selectMode = "";
var webcam = document.getElementById("webcam");
var coords = document.getElementById("coords");

function play_webcam() {
  webcam.setAttribute("src", "/tracked-stream");
  webcam.onclick = function () {
    pause_webcam();
  };
}

function pause_webcam() {
  webcam.setAttribute("src", "/tracked");
  webcam.onclick = function () {
    play_webcam();
  };
}

function setSelectMode(mode) {
  selectMode = mode;
  console.log(mode);
}

async function updatePosition(event) {
  let x = event.offsetX;
  let y = event.offsetY;
  let id = 0;

  const data = {
    x,
    y,
    id,
  };

  const response = await fetch("/update-position", {
    method: "POST",
    mode: "cors",
    cache: "no-cache",
    headers: {
      "Content-Type": "application/json",
    },
    redirect: "follow",
    referrerPolicy: "no-referrer",
    body: JSON.stringify(data),
  }).catch(console.log);
}

async function loadCoords() {
  const response = await fetch("/coords", {
    method: "GET",
    mode: "cors",
    cache: "no-cache",
    headers: {
      "Content-Type": "application/json",
    },
    redirect: "follow",
    referrerPolicy: "no-referrer",
  }).catch(console.log);

  coords.innerHTML = "";
  response.json().then((data) => {
    var text = document.createTextNode(JSON.stringify(data));
    coords.appendChild(text);
  });
}
