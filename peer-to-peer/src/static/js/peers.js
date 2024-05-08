function updatePeersTable(peers) {
    const peersTable = document.getElementById('peers');
    const rows = peers.map(peer => `
        <tr>
            <td>${peer}</td>
            <td><button onclick="sendCommand('ls r', { peerId: '${peer}' })">List Recipes</button></td>
        </tr>
    `).join('');
    peersTable.innerHTML = rows;
}

// Function to send command to the server
function sendCommand(command, payload) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ command, payload }));
        console.log(`Command sent: ${command}`, payload);
    } else {
        console.error("WebSocket is not connected.");
    }
}

// WebSocket setup
const ws = new WebSocket('ws://localhost:9000');

ws.onopen = function() {
    console.log('WebSocket connection successfully opened');
    // Fetch peers as soon as the WebSocket connection is opened
    sendCommand('ls p', {});
};

ws.onerror = function(error) {
    console.error('WebSocket Error:', error);
};

ws.onmessage = function(event) {
    console.log('Received message:', event.data);
    try {
        const data = JSON.parse(event.data);
        if (data.type === 'peers') {
            updatePeersTable(data.peers);
        } else {
            console.log('Received data of unknown type:', data);
        }
    } catch (error) {
        console.error('Error parsing received data:', error);
    }
};

document.addEventListener('DOMContentLoaded', () => {
    console.log('Peer management page loaded');
    // Optionally, fetch peers again if the WebSocket connection might not be ready when the page loads
    if (ws.readyState === WebSocket.OPEN) {
        sendCommand('ls p', {});
    } else {
        ws.addEventListener('open', () => {
            sendCommand('ls p', {});
        });
    }
});
