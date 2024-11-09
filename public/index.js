let clients = [];

function showChart (data) {
  const container = document.createElement("div");
  container.id = "d3-container";
  const containerElement = document.getElementById("container");
  containerElement.append(container);

  // Declare the chart dimensions and margins.
  const width = 640;
  const height = 400;
  const marginTop = 20;
  const marginRight = 20;
  const marginBottom = 30;
  const marginLeft = 40;

  // Declare the x (horizontal position) scale.
  const startDate = new Date(data[0][0] * 1000);
  const endDate = new Date(data[data.length - 1][0] * 1000);

  const x = d3.scaleUtc()
	.domain([startDate, endDate])
	.range([marginLeft, width - marginRight]);

  // Declare the y (vertical position) scale.
  const y = d3.scaleLinear()
	.domain([0, 100])
	.range([height - marginBottom, marginTop]);


  // Declare the line generator.
  const line = d3.line()
      .defined(d => !isNaN(d[1]))
      .x(d => x(new Date(d[0] * 1000)))
      .y(d => y(d[1]));

  // Create the SVG container.
  const svg = d3.create("svg")
	.attr("width", width)
	.attr("height", height);

  // Add the x-axis.
  svg.append("g")
	.attr("transform", `translate(0,${height - marginBottom})`)
	.call(d3.axisBottom(x));

  // Add the y-axis.
  svg.append("g")
	.attr("transform", `translate(${marginLeft},0)`)
	.call(d3.axisLeft(y));

  svg.append("path")
      .attr("fill", "none")
      .attr("stroke", "var(--foreground)")
      .attr("stroke-width", 1.5)
      .attr("d", line(data));

  // Append the SVG element.
  container.append(svg.node());
}

async function getClientData (id) {
  const res = await fetch(`/api/proxy/${id}`);
  return await res.json();
}

function getClientLoader(client) {
  return async function () {
    const containerElement = document.getElementById("container");
    containerElement.innerHTML = "";
    const data = await getClientData(client.id);
    console.log(data);
    const headerElement = document.createElement("h2");
    headerElement.textContent = client.address;
    const graphHeaderElement = document.createElement("h3");
    graphHeaderElement.textContent = "CPU Usage";
    const backElement = document.createElement("button");
    backElement.textContent = "Back";
    backElement.onclick = refreshClients;
    containerElement.appendChild(backElement);
    containerElement.appendChild(headerElement);
    containerElement.appendChild(graphHeaderElement);
    showChart(data.reverse());
  }
}

async function getClients () {
  const res = await fetch("/api/clients");
  return (await res.json()).clients;
}

async function refreshClients() {
  clients = await getClients();
  const containerElement = document.getElementById("container");
  containerElement.innerHTML = "";
  clients.forEach(c => {
    const clientElement = document.createElement("div");
    clientElement.textContent = c.address;
    clientElement.className = "client";
    clientElement.onclick = getClientLoader(c);
    containerElement.appendChild(clientElement);
  });
}

(async function main() {
  refreshClients();
})();
