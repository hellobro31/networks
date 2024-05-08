// Create WebSocket connection.
const ws = new WebSocket('ws://localhost:9000'); // Ensure this matches the backend URL and port
const responseContainer = document.getElementById('response');
const peersTable = document.getElementById('peers');
const recipesList = document.getElementById('recipes-list'); // Assuming there's an HTML element with this ID

// Connection opened
ws.onopen = function() {
    console.log('WebSocket connection successfully opened');
    sendCommand("ls p", {});  // Automatically request peers on connection
};

// Listen for errors
ws.onerror = function(error) {
    console.error('WebSocket Error:', error);
};

// Listen for messages
ws.onmessage = function(event) {
    console.log('Received message from backend:', event.data);
    try {
        const data = JSON.parse(event.data);
        if (data.type) {
            switch (data.type) {
                case 'peers':
                    updatePeersTable(data.peers);  // Assuming peers data is directly sent
                    break;
                case 'recipes':
                    updateRecipesList(data.recipes);  // Assuming recipes data is directly sent
                    break;
                default:
                    displayResponse('Received unknown data type from server.');
            }
        } else {
            displayResponse(data);  // Display any other message
        }
    } catch (error) {
        console.error('Error parsing received data:', error);
        displayResponse('Error in data received from server.');
    }
};

// Standardized function to send commands to the server as JSON
function sendCommand(command, payload) {
    const message = JSON.stringify({ command: command, ...payload });
    ws.send(message);
    console.log(`Command sent: ${message}`);
}

// Function to update peers table in the UI
function updatePeersTable(peers) {
    peersTable.innerHTML = peers.map(peer => `
        <tr>
            <td>${peer}</td>
            <td><button onclick="sendCommand('ls r', { peerId: '${peer}' })">List Recipes</button></td>
        </tr>
    `).join('');
}

// Function to update recipes list in the UI
function updateRecipesList(recipes) {
    recipesList.innerHTML = recipes.map(recipe => `
        <li>${recipe.name} - Ingredients: ${recipe.ingredients}, Instructions: ${recipe.instructions}</li>
    `).join('');
}

// Function to display any response or message in the UI
function displayResponse(message) {
    responseContainer.textContent = message;
}

// Function to handle creating a recipe
function createRecipe() {
    const name = document.getElementById('name').value;
    const ingredients = document.getElementById('ingredients').value;
    const instructions = document.getElementById('instructions').value;
    sendCommand("create r", { name, ingredients, instructions });
}

// Function to handle publishing a recipe
function publishRecipe() {
    const id = document.getElementById('publishId').value;
    sendCommand("publish r", { id });
}

document.getElementById('listRecipesBtn').addEventListener('click', function() {
    sendCommand("ls r all", {}); // Assuming there's a button with this ID for listing all recipes
});
