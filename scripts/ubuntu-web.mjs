// Local Ubuntu development reverse proxy. Not used by Replit's managed routing.
import http from 'node:http';
import net from 'node:net';

const server = http.createServer((req, res) => {
  const port = req.url?.startsWith('/api/') ? 5000 : 5173;
  const upstream = http.request({ hostname: '127.0.0.1', port, path: req.url, method: req.method, headers: req.headers }, (reply) => {
    res.writeHead(reply.statusCode ?? 502, reply.headers);
    reply.pipe(res);
  });
  upstream.on('error', () => { res.writeHead(502); res.end('Start the Sail API and frontend first.'); });
  req.pipe(upstream);
});
server.on('upgrade', (req, socket, head) => {
  const upstream = net.connect(5173, '127.0.0.1', () => {
    const headers = Object.entries(req.headers).map(([k, v]) => `${k}: ${Array.isArray(v) ? v.join(', ') : v}`).join('\r\n');
    upstream.write(`${req.method} ${req.url} HTTP/1.1\r\n${headers}\r\n\r\n`);
    if (head.length) upstream.write(head);
    socket.pipe(upstream).pipe(socket);
  });
  upstream.on('error', () => socket.destroy());
  socket.on('error', () => upstream.destroy());
});
server.listen(8080, '127.0.0.1', () => console.info('Sail local webpage: http://localhost:8080'));