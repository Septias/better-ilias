module.exports = {
    // **optional** default: `{}`
    // override vscode settings
    settings: {
        "vetur.useWorkspaceDependencies": true,
        "vetur.experimental.templateInterpolationService": true
    },
    // support monorepos
    projects: [
        './Frontend', // shorthand for only root.
        {
            // **required**
            // Where is your project?
            // It is relative to `vetur.config.js`.
            root: './Frontend',
        }
    ]
}