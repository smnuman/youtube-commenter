/**
 * YouTube Commenter - Main Application
 *
 * This file serves as the entry point for the YouTube Commenter application.
 * It handles user input, authentication, and displaying YouTube comments.
 */

// Import dependencies (to be installed)
// const { google } = require('googleapis');
// require('dotenv').config();

// Import local modules
const youtubeApi = require('./youtube-api');
const cli = require('./cli');

/**
 * Main function to run the application
 */
async function main() {
  console.log('YouTube Commenter');
  console.log('=================');
  console.log('This application will fetch and display comments from YouTube videos.');
  console.log('Note: Dependencies need to be installed before this will work properly.');

  try {
    // Get video ID from user
    const videoId = await cli.promptForVideoId();
    console.log(`\nFetching comments for video ID: ${videoId}\n`);

    // TODO: Once dependencies are installed:
    // 1. Authenticate with YouTube API
    // const youtube = youtubeApi.authenticate(process.env.YOUTUBE_API_KEY);
    //
    // 2. Fetch comments
    // const comments = await youtubeApi.fetchComments(youtube, videoId);
    //
    // 3. Display comments
    // cli.displayComments(comments);

    // For now, just show a placeholder message
    console.log('Comments would be displayed here once dependencies are installed.');
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Run the application
main().catch(console.error);
