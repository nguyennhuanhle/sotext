// SoText — Main Frontend Logic
// Communicates with Rust backend via Tauri invoke commands

const { invoke } = window.__TAURI__.core;

// ─── State ──────────────────────────────────────────────────
let currentFolder = '';
let currentResults = [];
let currentTemplatePath = null;
let sortDirection = { col: 'score', asc: false };
let selectedRow = null;
let currentLang = 'en';

// ─── DOM Elements ───────────────────────────────────────────
const folderPathInput = document.getElementById('folderPath');
const browseBtn = document.getElementById('browseBtn');
const scanBtn = document.getElementById('scanBtn');
const clearBtn = document.getElementById('clearBtn');
const thresholdInput = document.getElementById('threshold');
const ngramInput = document.getElementById('ngramSize');
const templateBtn = document.getElementById('templateBtn');
const exportCsvBtn = document.getElementById('exportCsvBtn');
const exportXlsxBtn = document.getElementById('exportXlsxBtn');
const exportHtmlBtn = document.getElementById('exportHtmlBtn');
const resultsBody = document.getElementById('resultsBody');
const emptyState = document.getElementById('emptyState');
const detailPanel = document.getElementById('detailPanel');
const detailLabelA = document.getElementById('detailLabelA');
const detailLabelB = document.getElementById('detailLabelB');
const detailMatchInfo = document.getElementById('detailMatchInfo');
const textA = document.getElementById('textA');
const textB = document.getElementById('textB');
const statusText = document.getElementById('statusText');
const progressBar = document.getElementById('progressBar');
const helpBtn = document.getElementById('helpBtn');
const helpModal = document.getElementById('helpModal');
const helpCloseBtn = document.getElementById('helpCloseBtn');
const helpTitle = document.getElementById('helpTitle');
const helpBody = document.getElementById('helpBody');
const langBtn = document.getElementById('langBtn');

// ─── i18n Helper ────────────────────────────────────────────

function t(key) { return i18n[currentLang][key]; }

function applyLanguage() {
  const L = i18n[currentLang];
  
  // Header
  document.querySelector('.app-subtitle').textContent = L.subtitle;
  helpBtn.textContent = L.helpBtn;
  langBtn.textContent = currentLang === 'en' ? '🌐 VI' : '🌐 EN';
  
  // Toolbar
  folderPathInput.placeholder = L.placeholder;
  browseBtn.textContent = L.browseBtn;
  scanBtn.innerHTML = `<span class="btn-icon">🔍</span> ${L.scanBtn.replace('🔍 ', '')}`;
  clearBtn.textContent = L.clearBtn;
  document.querySelector('label[for="threshold"]').textContent = L.thresholdLabel;
  document.querySelector('label[for="ngramSize"]').textContent = L.ngramLabel;
  if (!currentTemplatePath) templateBtn.textContent = L.templateBtn;
  exportCsvBtn.textContent = L.exportCsv;
  exportXlsxBtn.textContent = L.exportXlsx;
  exportHtmlBtn.textContent = L.exportHtml;
  
  // Status (only if in default state)
  if (!currentFolder) setStatus(L.statusReady);
  
  // Empty state
  emptyState.querySelector('p').textContent = L.emptyState;
  
  // Table headers
  const ths = document.querySelectorAll('#resultsTable th');
  ths[0].innerHTML = `${L.colFileA} <span class="sort-arrow"></span>`;
  ths[1].innerHTML = `${L.colFileB} <span class="sort-arrow"></span>`;
  ths[2].innerHTML = `${L.colScore} <span class="sort-arrow"></span>`;
  updateSortArrows();
  
  // Detail legend
  const legendItems = document.querySelectorAll('.legend-item');
  if (legendItems.length >= 2) {
    legendItems[0].innerHTML = `<span class="hl-dot hl-exact-dot"></span> ${L.legendExact}`;
    legendItems[1].innerHTML = `<span class="hl-dot hl-para-dot"></span> ${L.legendPara}`;
  }
  
  // Help modal
  helpTitle.textContent = L.helpTitle;
  helpBody.innerHTML = buildHelpHtml(L);
}

function buildHelpHtml(L) {
  return `
    <h3>${L.helpQuickStart}</h3>
    <ol>
      <li>${L.helpStep1}</li><li>${L.helpStep2}</li><li>${L.helpStep3}</li>
      <li>${L.helpStep4}</li><li>${L.helpStep5}</li><li>${L.helpStep6}</li><li>${L.helpStep7}</li>
    </ol>
    <h3>${L.helpParams}</h3>
    <table class="help-table">
      <tr><th>${L.helpParamHeader[0]}</th><th>${L.helpParamHeader[1]}</th><th>${L.helpParamHeader[2]}</th></tr>
      <tr><td><strong>Threshold</strong></td><td>${L.helpThresholdDesc}</td><td>30%</td></tr>
      <tr><td><strong>N-gram</strong></td><td>${L.helpNgramDesc}</td><td>5</td></tr>
    </table>
    <h3>${L.helpResults}</h3>
    <table class="help-table">
      <tr><th>${L.helpResultHeader[0]}</th><th>${L.helpResultHeader[1]}</th></tr>
      <tr><td><strong>Cosine Score</strong></td><td>${L.helpCosineDesc}</td></tr>
      <tr><td><strong>N-gram</strong></td><td>${L.helpNgramMatchDesc}</td></tr>
      <tr><td><strong>Suspicious</strong></td><td>${L.helpSuspiciousDesc}</td></tr>
      <tr><td><strong>Duplicates</strong></td><td>${L.helpDuplicateDesc}</td></tr>
    </table>
    <h3>${L.helpAlgorithms}</h3>
    <table class="help-table">
      <tr><th>${L.helpAlgoHeader[0]}</th><th>${L.helpAlgoHeader[1]}</th><th>${L.helpAlgoHeader[2]}</th></tr>
      <tr><td><strong>Level 1</strong></td><td>${L.helpAlgo1Name}</td><td>${L.helpAlgo1Catches}</td></tr>
      <tr><td><strong>Level 2</strong></td><td>${L.helpAlgo2Name}</td><td>${L.helpAlgo2Catches}</td></tr>
      <tr><td><strong>Level 3</strong></td><td>${L.helpAlgo3Name}</td><td>${L.helpAlgo3Catches}</td></tr>
      <tr><td><strong>Level 4</strong></td><td>${L.helpAlgo4Name}</td><td>${L.helpAlgo4Catches}</td></tr>
    </table>
    <h3>${L.helpFormats}</h3>
    <ul><li>${L.helpFormatTxt}</li><li>${L.helpFormatDocx}</li><li>${L.helpFormatHtml}</li></ul>
  `;
}

// ─── Event Listeners ────────────────────────────────────────

helpBtn.addEventListener('click', () => helpModal.classList.remove('hidden'));
helpCloseBtn.addEventListener('click', () => helpModal.classList.add('hidden'));
helpModal.addEventListener('click', (e) => {
  if (e.target === helpModal) helpModal.classList.add('hidden');
});

langBtn.addEventListener('click', () => {
  currentLang = currentLang === 'en' ? 'vi' : 'en';
  applyLanguage();
});

// Initialize language on load
applyLanguage();

browseBtn.addEventListener('click', async () => {
  try {
    const selected = await invoke('pick_folder');
    if (selected) {
      currentFolder = selected;
      folderPathInput.value = selected;
      const count = await invoke('count_files', { path: selected });
      setStatus(t('statusLoaded')(count));
    }
  } catch (err) {
    console.error('Browse error:', err);
  }
});

scanBtn.addEventListener('click', startScan);

clearBtn.addEventListener('click', () => {
  currentFolder = '';
  currentResults = [];
  currentTemplatePath = null;
  selectedRow = null;
  folderPathInput.value = '';
  resultsBody.innerHTML = '';
  emptyState.classList.remove('hidden');
  detailPanel.classList.add('hidden');
  exportCsvBtn.disabled = true;
  exportXlsxBtn.disabled = true;
  exportHtmlBtn.disabled = true;
  templateBtn.textContent = t('templateBtn');
  setStatus(t('statusReady'));
});

// Template upload
templateBtn.addEventListener('click', async () => {
  try {
    const selected = await invoke('pick_template_file');
    if (selected) {
      currentTemplatePath = selected;
      const fname = selected.split(/[\\/]/).pop();
      templateBtn.textContent = `📋 ${fname}`;
      setStatus(t('statusTemplateLoaded')(fname));
    }
  } catch (err) {
    console.error('Template error:', err);
  }
});

exportCsvBtn.addEventListener('click', async () => {
  if (currentResults.length === 0) return;
  try {
    const filepath = await invoke('pick_save_file', {
      defaultName: 'sotext_results.csv',
      filterName: 'CSV files',
      filterExt: 'csv',
    });
    if (filepath) {
      await invoke('export_csv', { results: currentResults, filepath });
      setStatus(t('statusExported')(filepath.split(/[\\/]/).pop()));
    }
  } catch (err) {
    setStatus(t('statusExportError')(err));
  }
});

exportXlsxBtn.addEventListener('click', async () => {
  if (currentResults.length === 0) return;
  try {
    const filepath = await invoke('pick_save_file', {
      defaultName: 'sotext_results.xlsx',
      filterName: 'Excel files',
      filterExt: 'xlsx',
    });
    if (filepath) {
      await invoke('export_excel', { results: currentResults, filepath });
      setStatus(t('statusExported')(filepath.split(/[\\/]/).pop()));
    }
  } catch (err) {
    setStatus(t('statusExportError')(err));
  }
});

exportHtmlBtn.addEventListener('click', async () => {
  if (currentResults.length === 0) return;
  try {
    const filepath = await invoke('pick_save_file', {
      defaultName: 'sotext_report.html',
      filterName: 'HTML files',
      filterExt: 'html',
    });
    if (filepath) {
      const ngramSize = parseInt(ngramInput.value) || 5;
      await invoke('export_html', {
        results: currentResults,
        folder: currentFolder,
        ngramSize,
        filepath,
      });
      setStatus(t('statusExported')(filepath.split(/[\\/]/).pop()));
    }
  } catch (err) {
    setStatus(t('statusExportError')(err));
  }
});

// Table header sorting
document.querySelectorAll('#resultsTable th.sortable').forEach(th => {
  th.addEventListener('click', () => {
    const col = th.dataset.col;
    if (sortDirection.col === col) {
      sortDirection.asc = !sortDirection.asc;
    } else {
      sortDirection.col = col;
      sortDirection.asc = col !== 'score';
    }
    sortAndRender();
  });
});

// ─── Scan ───────────────────────────────────────────────────

async function startScan() {
  const folder = currentFolder || folderPathInput.value.trim();
  if (!folder) {
    setStatus(t('statusNoFolder'));
    return;
  }

  const threshold = parseFloat(thresholdInput.value);
  if (isNaN(threshold) || threshold < 10 || threshold > 100) {
    setStatus(t('statusInvalidThreshold'));
    return;
  }

  // UI: scanning state
  scanBtn.disabled = true;
  scanBtn.classList.add('scanning');
  exportCsvBtn.disabled = true;
  exportXlsxBtn.disabled = true;
  exportHtmlBtn.disabled = true;
  progressBar.classList.remove('hidden');
  setStatus(t('statusScanning'));
  detailPanel.classList.add('hidden');

  try {
    const result = await invoke('scan_folder', {
      path: folder,
      threshold,
      templatePath: currentTemplatePath,
    });
    currentResults = result.pairs;

    setStatus(result.message);

    sortDirection = { col: 'score', asc: false };
    sortAndRender();

    if (currentResults.length > 0) {
      exportCsvBtn.disabled = false;
      exportXlsxBtn.disabled = false;
      exportHtmlBtn.disabled = false;
    }
  } catch (err) {
    setStatus(t('statusError')(err));
  } finally {
    scanBtn.disabled = false;
    scanBtn.classList.remove('scanning');
    progressBar.classList.add('hidden');
  }
}

// ─── Render Results ─────────────────────────────────────────

function sortAndRender() {
  const sorted = [...currentResults];
  const { col, asc } = sortDirection;

  sorted.sort((a, b) => {
    let va, vb;
    if (col === 'score') {
      va = a.score;
      vb = b.score;
    } else if (col === 'file_a') {
      va = a.file_a.toLowerCase();
      vb = b.file_a.toLowerCase();
    } else {
      va = a.file_b.toLowerCase();
      vb = b.file_b.toLowerCase();
    }

    if (va < vb) return asc ? -1 : 1;
    if (va > vb) return asc ? 1 : -1;
    return 0;
  });

  renderTable(sorted);
  updateSortArrows();
}

function renderTable(pairs) {
  resultsBody.innerHTML = '';

  if (pairs.length === 0) {
    emptyState.classList.remove('hidden');
    return;
  }

  emptyState.classList.add('hidden');

  pairs.forEach((pair, idx) => {
    const tr = document.createElement('tr');
    tr.style.animationDelay = `${idx * 0.03}s`;

    const scoreClass = pair.score >= 80 ? 'score-high' :
                       pair.score >= 60 ? 'score-medium' : 'score-low';

    tr.innerHTML = `
      <td title="${pair.file_a}">${pair.file_a}</td>
      <td title="${pair.file_b}">${pair.file_b}</td>
      <td class="${scoreClass}">${pair.score.toFixed(1)}%</td>
    `;

    tr.addEventListener('click', () => {
      if (selectedRow) selectedRow.classList.remove('selected');
      tr.classList.add('selected');
      selectedRow = tr;
      showDetail(pair);
    });

    resultsBody.appendChild(tr);
  });
}

function updateSortArrows() {
  document.querySelectorAll('#resultsTable th.sortable').forEach(th => {
    const arrow = th.querySelector('.sort-arrow');
    if (th.dataset.col === sortDirection.col) {
      arrow.textContent = sortDirection.asc ? '▲' : '▼';
    } else {
      arrow.textContent = '';
    }
  });
}

// ─── Detail View ────────────────────────────────────────────

async function showDetail(pair) {
  const folder = currentFolder || folderPathInput.value.trim();
  const ngramSize = parseInt(ngramInput.value) || 5;

  setStatus(t('statusDetailLoading'));

  try {
    const detail = await invoke('get_detail', {
      folder,
      fileA: pair.file_a,
      fileB: pair.file_b,
      ngramSize,
    });

    detailLabelA.textContent = `📄 ${detail.file_a}`;
    detailLabelB.textContent = `📄 ${detail.file_b}`;

    const susCount = detail.suspicious_sentences ? detail.suspicious_sentences.length : 0;
    detailMatchInfo.textContent = `${t('detailCosine')}: ${pair.score.toFixed(1)}% │ ${t('detailNgrams')}: ${detail.common_phrase_count} │ ${t('detailSuspicious')}: ${susCount}`;

    textA.innerHTML = highlightText(detail.content_a, detail.highlights_a, detail.suspicious_sentences, true);
    textB.innerHTML = highlightText(detail.content_b, detail.highlights_b, detail.suspicious_sentences, false);

    detailPanel.classList.remove('hidden');
    setStatus(t('statusReady'));
  } catch (err) {
    setStatus(t('statusDetailError')(err));
  }
}

function highlightText(content, ngramRanges, suspiciousSentences, isA) {
  // Collect all ranges: { start, end, type }
  const allRanges = [];

  // N-gram exact matches
  if (ngramRanges) {
    for (const [start, end] of ngramRanges) {
      allRanges.push({ start, end, type: 'exact' });
    }
  }

  // Suspicious sentence ranges
  if (suspiciousSentences) {
    for (const pair of suspiciousSentences) {
      const [s, e] = isA ? pair.pos_a : pair.pos_b;
      // Only add if not already covered by exact match
      const covered = ngramRanges && ngramRanges.some(([ns, ne]) => ns <= s && ne >= e);
      if (!covered) {
        allRanges.push({ start: s, end: e, type: 'para' });
      }
    }
  }

  if (allRanges.length === 0) {
    return escapeHtml(content);
  }

  // Sort by start position
  allRanges.sort((a, b) => a.start - b.start);

  let html = '';
  let lastEnd = 0;

  for (const range of allRanges) {
    const start = range.start;
    const end = Math.min(range.end, content.length);
    if (start < lastEnd) continue; // skip overlapping

    html += escapeHtml(content.substring(lastEnd, start));
    const cls = range.type === 'exact' ? 'highlight-match' : 'highlight-para';
    html += `<span class="${cls}">${escapeHtml(content.substring(start, end))}</span>`;
    lastEnd = end;
  }

  html += escapeHtml(content.substring(lastEnd));
  return html;
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ─── Status ─────────────────────────────────────────────────

function setStatus(text) {
  statusText.textContent = text;
}
