// Assuming ws is the WebSocket connection already established
const ws = new WebSocket('ws://localhost:9000');

// Function to create a new recipe and send it to the backend using JSON
function createRecipe() {
    const name = document.getElementById('name').value;
    const ingredients = document.getElementById('ingredients').value;
    const instructions = document.getElementById('instructions').value;
    console.log('Creating recipe:', name, ingredients, instructions);
    
    // Constructing the recipe object
    const recipeData = {
        command: 'create r',
        name: name,
        ingredients: ingredients,
        instructions: instructions
    };
    
    // Sending the create recipe command to the server as a JSON string
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(`${recipeData.command} ${recipeData.name}|${recipeData.ingredients}|${recipeData.instructions}`);  // Sending as JSON
        console.log('Recipe data sent to the server:', recipeData);
    } else {
        console.error("WebSocket is not connected.");
    }
}

// Function to request a list of all recipes from the backend
function listAllRecipes() {
    console.log('Listing all recipes');
    
    // Send the command to list all recipes
    const command = 'ls r all';
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(command);
        console.log('Request sent to list all recipes.');
    } else {
        console.error("WebSocket is not connected.");
    }
}

// Function to publish a recipe
function publishRecipe() {
    const id = document.getElementById('recipeId').value;
    console.log('Publishing recipe:', id);

    const command = {
        command: 'publish r',
        id: id
    };

    if (ws.readyState === WebSocket.OPEN) {
        ws.send(`${command.command} ${command.id}`);  // Sending as JSON
        console.log('Publish command sent for recipe ID:', id);
    } else {
        console.error("WebSocket is not connected.");
    }
}

// Function to request a list of all active peers
function listPeers() {
    console.log('Listing all peers');

    const command = { command: 'ls p' };
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(`${command.command}`);
        console.log('Request sent to list all peers.');
    } else {
        console.error("WebSocket is not connected.");
    }
}

// Handle incoming messages from the WebSocket connection
ws.onmessage = function(event) {
    console.log('Received message:', event.data);
    try {
        const data = JSON.parse(event.data);
        // Handling different responses based on data.command or type
        if (data.command === 'update recipes list') {
            updateRecipesList(data.recipes);
        } else if (data.command === 'update peers list') {
            updatePeersList(data.peers);
        }
    } catch (error) {
        console.error('Error parsing received data:', error);
    }
};

// Function to update the DOM with a list of recipes
function updateRecipesList(recipes) {
    const recipesList = document.getElementById('recipes-list');
    recipesList.innerHTML = ''; // Clear the list first
    recipes.forEach(recipe => {
        const li = document.createElement('li');
        li.textContent = `${recipe.name} - Ingredients: ${recipe.ingredients}, Instructions: ${recipe.instructions}`;
        recipesList.appendChild(li);
    });
    console.log('Recipes list updated.');
}

// Function to update the DOM with a list of peers
function updatePeersList(peers) {
    const peersList = document.getElementById('peers-list');
    peersList.innerHTML = ''; // Clear the list first
    peers.forEach(peer => {
        const li = document.createElement('li');
        li.textContent = `Peer ID: ${peer.id}`;
        peersList.appendChild(li);
    });
    console.log('Peers list updated.');
}

document.addEventListener('DOMContentLoaded', () => {
    const createButton = document.getElementById('createButton');
    const listRecipesButton = document.getElementById('listButton');
    const publishButton = document.getElementById('publishButton');
    const listPeersButton = document.getElementById('listPeersButton');

    // Add event listeners for buttons
    createButton && createButton.addEventListener('click', createRecipe);
    listRecipesButton && listRecipesButton.addEventListener('click', listAllRecipes);
    publishButton && publishButton.addEventListener('click', publishRecipe);
    listPeersButton && listPeersButton.addEventListener('click', listPeers);
});
