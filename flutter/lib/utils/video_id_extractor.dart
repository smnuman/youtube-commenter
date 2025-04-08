/// Extract YouTube video ID from a URL
String? extractVideoId(String input) {
  // Handle direct video IDs (11 characters)
  if (input.length == 11 && !input.contains('/') && !input.contains('?')) {
    return input;
  }
  
  // Try to parse as URL
  Uri? uri;
  try {
    uri = Uri.parse(input);
  } catch (_) {
    return null;
  }
  
  // Handle youtu.be format
  if (uri.host == 'youtu.be') {
    return uri.pathSegments.isNotEmpty ? uri.pathSegments.first : null;
  }
  
  // Handle youtube.com format
  if (uri.host == 'youtube.com' || uri.host == 'www.youtube.com') {
    return uri.queryParameters['v'];
  }
  
  return null;
}
