/**
 * Command Line Interface for YouTube Commenter
 * 
 * This module handles the command line interface for the application.
 */

// Import dependencies (to be installed)
// const inquirer = require('inquirer');

/**
 * Prompt the user for a YouTube video URL or ID
 * @returns {Promise<string>} - The YouTube video ID
 */
async function promptForVideoId() {
  // This will be implemented once dependencies are installed
  // const answers = await inquirer.prompt([
  //   {
  //     type: 'input',
  //     name: 'videoUrl',
  //     message: 'Enter a YouTube video URL or ID:',
  //     validate: (input) => {
  //       if (input.trim() === '') {
  //         return 'Please enter a valid YouTube video URL or ID';
  //       }
  //       return true;
  //     }
  //   }
  // ]);
  
  // return extractVideoId(answers.videoUrl);
  
  // For now, just return a placeholder
  console.log('Enter a YouTube video URL or ID:');
  return 'dQw4w9WgXcQ'; // Example video ID (Rick Astley - Never Gonna Give You Up)
}

/**
 * Extract the video ID from a YouTube URL
 * @param {string} input - YouTube URL or video ID
 * @returns {string} - YouTube video ID
 */
function extractVideoId(input) {
  // Handle direct video IDs
  if (input.length === 11) {
    return input;
  }
  
  // Handle YouTube URLs
  try {
    const url = new URL(input);
    if (url.hostname === 'youtu.be') {
      return url.pathname.substring(1);
    }
    if (url.hostname === 'www.youtube.com' || url.hostname === 'youtube.com') {
      const params = new URLSearchParams(url.search);
      return params.get('v');
    }
  } catch (error) {
    // Not a valid URL, might be a direct video ID
    return input;
  }
  
  // Default case, return as is
  return input;
}

/**
 * Display comments in a readable format
 * @param {Array} comments - List of comments
 */
function displayComments(comments) {
  if (!comments || comments.length === 0) {
    console.log('No comments found.');
    return;
  }
  
  console.log(`\nFound ${comments.length} comments:\n`);
  
  comments.forEach((comment, index) => {
    // This will be implemented once we have actual comment data
    console.log(`Comment #${index + 1}: Placeholder comment`);
  });
}

module.exports = {
  promptForVideoId,
  extractVideoId,
  displayComments
};
