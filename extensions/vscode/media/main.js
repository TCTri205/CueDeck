// @ts-ignore
const vscode = acquireVsCodeApi();

let cy = null;

// Initialize Cytoscape when the page loads
document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing Cytoscape...');

    // @ts-ignore
    cy = cytoscape({
        container: document.getElementById('cy'),

        style: [
            {
                selector: 'node',
                style: {
                    'background-color': '#3794ff',
                    'label': 'data(label)',
                    'color': '#ffffff',
                    'text-valign': 'center',
                    'text-halign': 'center',
                    'text-wrap': 'wrap',
                    'text-max-width': '100px',
                    'font-size': '14px',
                    'font-weight': 'bold',
                    'text-outline-width': 2,
                    'text-outline-color': '#000000',
                    'width': '80px',
                    'height': '80px',
                    'border-width': 3,
                    'border-color': '#0078d4'
                }
            },
            {
                selector: 'edge',
                style: {
                    'width': 2,
                    'line-color': '#666666',
                    'target-arrow-color': '#666666',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'label': 'data(label)',
                    'font-size': '10px',
                    'color': '#999999'
                }
            },
            {
                selector: 'node:selected',
                style: {
                    'background-color': '#0078d4',
                    'border-color': '#005a9e',
                    'border-width': 4,
                    'color': '#ffff00',
                    'text-outline-color': '#000000'
                }
            },
            {
                selector: 'node:hover',
                style: {
                    'background-color': '#2b88d8',
                    'cursor': 'pointer'
                }
            },
            // Status-based colors
            {
                selector: 'node[status="todo"]',
                style: {
                    'background-color': '#3794ff'
                }
            },
            {
                selector: 'node[status="active"]',
                style: {
                    'background-color': '#ff9500'
                }
            },
            {
                selector: 'node[status="done"]',
                style: {
                    'background-color': '#28a745'
                }
            },
            {
                selector: 'node[status="archived"]',
                style: {
                    'background-color': '#6c757d'
                }
            },
            // Priority-based borders
            {
                selector: 'node[priority="critical"]',
                style: {
                    'border-color': '#dc3545',
                    'border-width': 5
                }
            },
            {
                selector: 'node[priority="high"]',
                style: {
                    'border-color': '#ff9500',
                    'border-width': 4
                }
            },
            {
                selector: 'node[priority="low"]',
                style: {
                    'border-color': '#28a745',
                    'border-width': 2
                }
            }
        ],

        layout: {
            name: 'cose',
            animate: true,
            animationDuration: 500,
            nodeRepulsion: 10000,
            idealEdgeLength: 150,
            edgeElasticity: 100,
            nestingFactor: 5,
            gravity: 1,
            numIter: 2000,
            initialTemp: 200,
            coolingFactor: 0.95,
            minTemp: 1.0
        },

        // Enable panning and zooming
        wheelSensitivity: 0.2,
        minZoom: 0.1,
        maxZoom: 5
    });

    console.log('Cytoscape initialized:', cy);

    // Handle node clicks - open file in editor
    cy.on('tap', 'node', function (evt) {
        const node = evt.target;
        const filePath = node.data('path');
        const label = node.data('label');
        console.log('Node clicked:', label, filePath);
        if (filePath) {
            vscode.postMessage({
                command: 'openFile',
                path: filePath
            });
        }
    });

    // Add tooltip-like behavior using status bar
    cy.on('mouseover', 'node', function (evt) {
        const node = evt.target;
        const label = node.data('label');
        const status = node.data('status') || 'unknown';
        const priority = node.data('priority') || 'medium';

        // Update status bar with node info
        const statusEl = document.getElementById('status');
        if (statusEl) {
            statusEl.textContent = `📄 ${label} | Status: ${status} | Priority: ${priority}`;
        }
    });

    cy.on('mouseout', 'node', function (evt) {
        // Restore default status
        const statusEl = document.getElementById('status');
        if (statusEl && cy) {
            const stats = cy.elements();
            statusEl.textContent = `Loaded ${stats.nodes().length} nodes, ${stats.edges().length} edges`;
        }
    });
});

// Refresh button
document.getElementById('refresh-btn')?.addEventListener('click', () => {
    console.log('Refresh button clicked');
    vscode.postMessage({ command: 'refresh' });
    const statusEl = document.getElementById('status');
    if (statusEl) {
        statusEl.textContent = 'Loading...';
    }
});

// Fit to screen button
document.getElementById('fit-btn')?.addEventListener('click', () => {
    console.log('Fit button clicked');
    if (cy) {
        cy.fit(null, 50);
    }
});

// Function to run layout with selected algorithm
function runLayout(layoutName) {
    if (!cy) {
        return;
    }

    const layoutConfigs = {
        cose: {
            name: 'cose',
            animate: true,
            animationDuration: 500,
            nodeRepulsion: 10000,
            idealEdgeLength: 150,
            edgeElasticity: 100,
            nestingFactor: 5,
            gravity: 1,
            numIter: 2000,
            initialTemp: 200,
            coolingFactor: 0.95,
            minTemp: 1.0
        },
        dagre: {
            name: 'dagre',
            rankDir: 'TB', // Top to bottom
            animate: true,
            animationDuration: 500
        },
        circle: {
            name: 'circle',
            animate: true,
            animationDuration: 500
        },
        grid: {
            name: 'grid',
            animate: true,
            animationDuration: 500
        },
        breadthfirst: {
            name: 'breadthfirst',
            directed: true,
            animate: true,
            animationDuration: 500
        }
    };

    const config = layoutConfigs[layoutName] || layoutConfigs.cose;
    console.log('Running layout:', layoutName, config);
    cy.layout(config).run();
}

// Re-layout button - uses selected layout
document.getElementById('layout-btn')?.addEventListener('click', () => {
    console.log('Re-layout button clicked');
    const selector = document.getElementById('layout-selector');
    const layout = selector ? selector.value : 'cose';
    runLayout(layout);
});

// Layout selector dropdown
document.getElementById('layout-selector')?.addEventListener('change', (e) => {
    const layout = e.target.value;
    console.log('Layout changed to:', layout);
    runLayout(layout);
});

// Handle messages from extension
window.addEventListener('message', event => {
    const message = event.data;
    console.log('Message received:', message.command);

    switch (message.command) {
        case 'graphData':
            const data = message.data;
            console.log('Graph data received:', {
                nodeCount: data.nodes.length,
                edgeCount: data.edges.length,
                stats: data.stats,
                sampleNode: data.nodes[0]
            });

            // Update status
            const statusEl = document.getElementById('status');
            if (statusEl) {
                statusEl.textContent = `Loaded ${data.stats.node_count} nodes, ${data.stats.edge_count} edges`;
            }

            // Clear existing graph
            if (cy) {
                cy.elements().remove();

                // Add nodes with metadata for styling
                const nodes = data.nodes.map(node => {
                    const nodeData = {
                        group: 'nodes',
                        data: {
                            id: 'n' + node.id,
                            label: node.name,
                            path: node.path,
                            status: node.metadata?.status || 'todo',
                            priority: node.metadata?.priority || 'medium',
                            type: node.type || 'document'
                        }
                    };
                    console.log('Creating node:', nodeData);
                    return nodeData;
                });

                // Add edges
                const edges = data.edges.map((edge, idx) => ({
                    group: 'edges',
                    data: {
                        id: 'e' + idx,
                        source: 'n' + edge.from,
                        target: 'n' + edge.to
                    }
                }));

                // Add all elements
                cy.add(nodes);
                cy.add(edges);

                console.log(`Successfully added ${nodes.length} nodes and ${edges.length} edges to graph`);
                console.log('First node in graph:', cy.nodes()[0]?.data());

                // Run layout
                const layout = cy.layout({
                    name: 'cose',
                    animate: true,
                    animationDuration: 1000,
                    nodeRepulsion: 10000,
                    idealEdgeLength: 150,
                    edgeElasticity: 100,
                    nestingFactor: 5,
                    gravity: 1,
                    numIter: 2000,
                    initialTemp: 200,
                    coolingFactor: 0.95,
                    minTemp: 1.0
                });

                layout.run();

                // Fit to screen after layout completes
                layout.one('layoutstop', () => {
                    console.log('Layout completed');
                    setTimeout(() => {
                        cy.fit(null, 50);
                        console.log('Fitted to screen');
                    }, 100);
                });
            }
            break;

        case 'error':
            console.error('Error from extension:', message.error);
            const statusElError = document.getElementById('status');
            if (statusElError) {
                statusElError.textContent = `Error: ${message.error}`;
            }
            break;
    }
});
