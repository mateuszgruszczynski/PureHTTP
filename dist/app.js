const { invoke } = window.__TAURI__.core;

let startTime;

// DOM elements
const methodSelect = document.getElementById('method');
const urlInput = document.getElementById('url');
const headersTextarea = document.getElementById('headers');
const bodyTextarea = document.getElementById('body');
const executeBtn = document.getElementById('execute-btn');
const saveBtn = document.getElementById('save-btn');
const loadBtn = document.getElementById('load-btn');
const prettifyBodyBtn = document.getElementById('prettify-body-btn');
const prettifyResponseBtn = document.getElementById('prettify-response-btn');

const statusCodeSpan = document.getElementById('status-code');
const responseTimeSpan = document.getElementById('response-time');
const responseHeadersTextarea = document.getElementById('response-headers');
const responseTextarea = document.getElementById('response');

// Execute request
executeBtn.addEventListener('click', async () => {
    if (!urlInput.value.trim()) {
        alert('Please enter a URL');
        return;
    }

    executeBtn.disabled = true;
    executeBtn.textContent = 'Executing...';
    
    // Clear previous response
    statusCodeSpan.textContent = '';
    responseTimeSpan.textContent = '';
    responseHeadersTextarea.value = '';
    responseTextarea.value = '';
    
    startTime = Date.now();
    
    try {
        const response = await invoke('execute_request', {
            method: methodSelect.value,
            url: urlInput.value.trim(),
            headers: headersTextarea.value,
            body: bodyTextarea.value || null
        });
        
        const endTime = Date.now();
        const duration = endTime - startTime;
        
        // Update status
        statusCodeSpan.textContent = `${response.status} ${response.status_text}`;
        statusCodeSpan.className = `status-code ${response.status >= 200 && response.status < 300 ? 'success' : 'error'}`;
        responseTimeSpan.textContent = `${duration}ms`;
        
        // Update response
        responseHeadersTextarea.value = response.headers;
        
        if (typeof response.body === 'object') {
            responseTextarea.value = JSON.stringify(response.body, null, 2);
        } else {
            responseTextarea.value = response.body;
        }
        
    } catch (error) {
        statusCodeSpan.textContent = 'ERROR';
        statusCodeSpan.className = 'status-code error';
        responseTextarea.value = `Error: ${error}`;
    }
    
    executeBtn.disabled = false;
    executeBtn.textContent = 'Execute';
});

// Save request
saveBtn.addEventListener('click', async () => {
    const request = {
        method: methodSelect.value,
        url: urlInput.value,
        headers: headersTextarea.value,
        body: bodyTextarea.value
    };
    
    try {
        await invoke('save_request', {
            request: JSON.stringify(request, null, 2)
        });
    } catch (error) {
        alert(`Save failed: ${error}`);
    }
});

// Load request
loadBtn.addEventListener('click', async () => {
    try {
        const requestStr = await invoke('load_request');
        const request = JSON.parse(requestStr);
        
        methodSelect.value = request.method || 'GET';
        urlInput.value = request.url || '';
        headersTextarea.value = request.headers || '';
        bodyTextarea.value = request.body || '';
        
    } catch (error) {
        if (!error.includes('cancelled')) {
            alert(`Load failed: ${error}`);
        }
    }
});

// Prettify JSON/XML
function prettifyJson(text) {
    try {
        const parsed = JSON.parse(text);
        return JSON.stringify(parsed, null, 2);
    } catch {
        return text;
    }
}

prettifyBodyBtn.addEventListener('click', () => {
    bodyTextarea.value = prettifyJson(bodyTextarea.value);
});

prettifyResponseBtn.addEventListener('click', () => {
    responseTextarea.value = prettifyJson(responseTextarea.value);
});

// Handle Enter key in URL input
urlInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
        executeBtn.click();
    }
});