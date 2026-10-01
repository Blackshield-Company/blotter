function showError(message) {
  const el = document.getElementById("error");
  el.hidden = !message;
  el.textContent = message || "";
}

const invoke = window.__TAURI__ && window.__TAURI__.core
  ? window.__TAURI__.core.invoke
  : null;

if (!invoke) {
  showError("Open this in the Blotter window.");
}

async function call(cmd, args) {
  if (!invoke) {
    showError("Open this in the Blotter window.");
    throw new Error("Open this in the Blotter window.");
  }
  try {
    return await invoke(cmd, args || {});
  } catch (err) {
    const message = typeof err === "string" ? err : (err && err.message) || String(err);
    showError(message);
    throw err;
  }
}

function field(form, name) {
  return form.elements[name].value.trim();
}

function appendCounts(root, title, counts) {
  const heading = document.createElement("h2");
  heading.textContent = title;
  root.appendChild(heading);
  const list = document.createElement("ul");
  list.className = "reminders";
  const entries = Object.entries(counts || {});
  entries.sort((a, b) => b[1] - a[1] || String(a[0]).localeCompare(String(b[0])));
  if (!entries.length) {
    const empty = document.createElement("li");
    empty.textContent = "None.";
    list.appendChild(empty);
  } else {
    for (const [name, count] of entries) {
      const item = document.createElement("li");
      item.textContent = name + "  " + count;
      list.appendChild(item);
    }
  }
  root.appendChild(list);
}

function appendTable(root, records) {
  const heading = document.createElement("h2");
  heading.textContent = "Rows";
  root.appendChild(heading);
  const table = document.createElement("table");
  const head = document.createElement("thead");
  const headRow = document.createElement("tr");
  for (const label of ["id", "datetime", "category", "district", "description"]) {
    const cell = document.createElement("th");
    cell.textContent = label;
    headRow.appendChild(cell);
  }
  head.appendChild(headRow);
  table.appendChild(head);
  const body = document.createElement("tbody");
  for (const rec of records) {
    const row = document.createElement("tr");
    const cells = [
      rec.incident_id || "",
      rec.datetime || "",
      rec.category || "",
      rec.district_or_neighborhood || "",
      rec.description || "",
    ];
    for (const value of cells) {
      const cell = document.createElement("td");
      cell.textContent = value;
      row.appendChild(cell);
    }
    body.appendChild(row);
  }
  if (!records.length) {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    cell.colSpan = 5;
    cell.textContent = "No rows.";
    row.appendChild(cell);
    body.appendChild(row);
  }
  table.appendChild(body);
  root.appendChild(table);
}

function renderDataset(data) {
  const root = document.getElementById("detail");
  root.replaceChildren();
  const stats = (data && data.stats) || {};
  const total = document.createElement("p");
  total.textContent = "Total: " + (stats.total || 0);
  root.appendChild(total);
  appendCounts(root, "By category", stats.by_category);
  appendCounts(root, "By month", stats.by_month);
  appendCounts(root, "By district", stats.by_district);
  appendTable(root, (data && data.records) || []);
}

function renderSkipped(skipped) {
  const box = document.getElementById("skipped");
  box.replaceChildren();
  if (!skipped || !skipped.length) return;
  const heading = document.createElement("h2");
  heading.textContent = "Skipped";
  const list = document.createElement("ul");
  list.className = "reminders";
  for (const item of skipped) {
    const row = document.createElement("li");
    const num = Array.isArray(item) ? item[0] : "";
    const reason = Array.isArray(item) ? item[1] : String(item);
    row.textContent = "row " + num + ": " + reason;
    list.appendChild(row);
  }
  box.append(heading, list);
}

async function onNormalize(form) {
  showError("");
  const result = await call("normalize", {
    configPath: field(form, "config_path"),
    inputPath: field(form, "input_path"),
    outputPath: field(form, "output_path"),
  });
  document.getElementById("result").textContent =
    "Wrote " + result.written + " rows to " + result.output_path;
  renderSkipped(result.skipped);
  const loaded = await call("load", { path: result.output_path });
  renderDataset(loaded);
}

async function onLoad(form) {
  showError("");
  renderSkipped([]);
  document.getElementById("result").textContent = "";
  const loaded = await call("load", { path: field(form, "path") });
  document.getElementById("result").textContent = "Loaded " + loaded.stats.total + " rows.";
  renderDataset(loaded);
}

document.addEventListener("submit", async (event) => {
  if (event.target.matches("#normalize-form")) {
    event.preventDefault();
    try {
      await onNormalize(event.target);
    } catch (_err) {
      /* shown in the error bar */
    }
  }
  if (event.target.matches("#load-form")) {
    event.preventDefault();
    try {
      await onLoad(event.target);
    } catch (_err) {
      /* shown in the error bar */
    }
  }
});


