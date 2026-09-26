const fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const manifest=JSON.parse(fs.readFileSync(path.join(root,'manifest.json'),'utf8'));
const palette=fs.readFileSync(path.join(root,'engineer-16.pal'),'utf8').trim().split(/\r?\n/).slice(3).map(s=>s.split(' ').map(n=>Number(n).toString(16).padStart(2,'0')).join(''));
fs.writeFileSync(path.join(root,'preview','preview-data.js'),'window.ENGINEER_DATA = '+JSON.stringify({frames:manifest.frames,palette})+';\n');
console.log('Packaged offline preview metadata.');
