// Initialize WebView functionality
document.addEventListener('DOMContentLoaded', () => {
    // Set up link handling
    document.querySelectorAll('a[href^="http"]').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            window.open(link.href, '_blank');
        });
    });

    // Handle keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Close article view with Escape
        if (e.key === 'Escape') {
            window.close();
        }
        
        // Toggle dark mode with Ctrl/Cmd + D
        if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
            e.preventDefault();
            document.documentElement.classList.toggle('dark-mode');
        }
    });

    // Apply system theme preference
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.classList.add('dark-mode');
    }

    // Listen for system theme changes
    window.matchMedia('(prefers-color-scheme: dark)').addListener(e => {
        if (e.matches) {
            document.documentElement.classList.add('dark-mode');
        } else {
            document.documentElement.classList.remove('dark-mode');
        }
    });

    // Set up image handling
    document.querySelectorAll('img').forEach(img => {
        // Add loading="lazy" for better performance
        img.loading = 'lazy';
        
        // Add click handler for full-size image view
        img.addEventListener('click', () => {
            const fullSize = window.open('', '_blank');
            fullSize.document.write(`
                <html>
                <head>
                    <style>
                        body {
                            margin: 0;
                            display: flex;
                            justify-content: center;
                            align-items: center;
                            min-height: 100vh;
                            background: #000;
                        }
                        img {
                            max-width: 100%;
                            max-height: 100vh;
                            object-fit: contain;
                        }
                    </style>
                </head>
                <body>
                    <img src="${img.src}" alt="${img.alt || ''}">
                </body>
                </html>
            `);
        });
    });

    // Initialize smooth scrolling
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', (e) => {
            e.preventDefault();
            const target = document.querySelector(anchor.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Set up content zoom controls
    let currentZoom = 100;
    
    function updateZoom(delta) {
        currentZoom = Math.max(50, Math.min(200, currentZoom + delta));
        document.body.style.zoom = `${currentZoom}%`;
    }

    document.addEventListener('keydown', (e) => {
        // Zoom in/out with Ctrl/Cmd + +/-
        if (e.ctrlKey || e.metaKey) {
            if (e.key === '=' || e.key === '+') {
                e.preventDefault();
                updateZoom(10);
            } else if (e.key === '-') {
                e.preventDefault();
                updateZoom(-10);
            } else if (e.key === '0') {
                e.preventDefault();
                currentZoom = 100;
                document.body.style.zoom = '100%';
            }
        }
    });

    // Handle wheel zooming
    document.addEventListener('wheel', (e) => {
        if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            updateZoom(e.deltaY > 0 ? -5 : 5);
        }
    });
});