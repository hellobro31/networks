const ws = new WebSocket('ws://localhost:9000');

ws.onopen = () => {
    console.log('WebSocket connection opened.');
};

ws.onerror = error => {
    console.error('WebSocket error:', error);
};

ws.onmessage = event => {
    console.log('Received message:', event.data);
    const messageBox = document.getElementById('message');
    messageBox.textContent = `Server says: ${event.data}`;
};

document.getElementById('publishButton').addEventListener('click', () => {
    const recipeId = document.getElementById('recipeId').value;
    if (recipeId) {
        ws.send(JSON.stringify({ command: 'publish r', id: recipeId }));
        console.log('Publish command sent for recipe ID:', recipeId);
    } else {
        alert('Please enter a recipe ID.');
    }
});
