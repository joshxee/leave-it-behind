// Optional local-only review server. Run from the repository root.
const http = require('node:http'), fs = require('node:fs'), path = require('node:path');
const root = path.resolve(__dirname, '../..');
http.createServer((req, res) => {
  let file;
  try { file = path.resolve(root, '.' + decodeURIComponent(new URL(req.url, 'http://localhost').pathname)); }
  catch { res.writeHead(400).end(); return; }
  if (file === root) file = path.join(root, 'maintenance/index.html');
  if (!file.startsWith(root + path.sep)) { res.writeHead(403).end(); return; }
  fs.readFile(file, (error, data) => {
    if (error) { res.writeHead(404).end(); return; }
    res.setHeader('Content-Type', ({ '.html': 'text/html', '.png': 'image/png', '.json': 'application/json', '.md': 'text/plain' })[path.extname(file)] || 'application/octet-stream');
    res.end(data);
  });
}).listen(4176, '127.0.0.1', () => console.log('Maintenance review: http://127.0.0.1:4176/maintenance/index.html'));
