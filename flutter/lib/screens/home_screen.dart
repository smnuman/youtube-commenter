import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';

import '../main.dart';
import '../models/comment.dart';
import '../utils/video_id_extractor.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final TextEditingController _videoUrlController = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void dispose() {
    _videoUrlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final commentProvider = Provider.of<CommentProvider>(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('YouTube Commenter'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Form(
              key: _formKey,
              child: Row(
                children: [
                  Expanded(
                    child: TextFormField(
                      controller: _videoUrlController,
                      decoration: const InputDecoration(
                        labelText: 'YouTube Video URL or ID',
                        hintText: 'https://www.youtube.com/watch?v=...',
                        border: OutlineInputBorder(),
                      ),
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return 'Please enter a YouTube video URL or ID';
                        }
                        final videoId = extractVideoId(value);
                        if (videoId == null) {
                          return 'Invalid YouTube URL or ID';
                        }
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(width: 16),
                  ElevatedButton(
                    onPressed: () {
                      if (_formKey.currentState!.validate()) {
                        final videoId = extractVideoId(_videoUrlController.text);
                        if (videoId != null) {
                          commentProvider.fetchComments(videoId);
                        }
                      }
                    },
                    child: const Text('Fetch Comments'),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            if (commentProvider.isLoading)
              const Center(child: CircularProgressIndicator())
            else if (commentProvider.error != null)
              Center(
                child: Text(
                  'Error: ${commentProvider.error}',
                  style: const TextStyle(color: Colors.red),
                ),
              )
            else if (commentProvider.comments.isEmpty)
              const Center(
                child: Text('No comments to display. Enter a YouTube URL above.'),
              )
            else
              Expanded(
                child: CommentList(comments: commentProvider.comments),
              ),
          ],
        ),
      ),
    );
  }
}

class CommentList extends StatelessWidget {
  final List<Comment> comments;

  const CommentList({super.key, required this.comments});

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: comments.length,
      itemBuilder: (context, index) {
        final comment = comments[index];
        return CommentCard(comment: comment);
      },
    );
  }
}

class CommentCard extends StatefulWidget {
  final Comment comment;

  const CommentCard({super.key, required this.comment});

  @override
  State<CommentCard> createState() => _CommentCardState();
}

class _CommentCardState extends State<CommentCard> {
  bool _showReplies = false;

  @override
  Widget build(BuildContext context) {
    final dateFormat = DateFormat('MMM d, yyyy');
    
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 8),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                CircleAvatar(
                  backgroundColor: Colors.grey.shade200,
                  child: Text(widget.comment.author[0]),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            widget.comment.author,
                            style: const TextStyle(fontWeight: FontWeight.bold),
                          ),
                          Text(
                            dateFormat.format(widget.comment.publishedAt),
                            style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
                          ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Text(widget.comment.text),
                      const SizedBox(height: 8),
                      Row(
                        children: [
                          const Icon(Icons.thumb_up_outlined, size: 16),
                          const SizedBox(width: 4),
                          Text('${widget.comment.likeCount}'),
                          if (widget.comment.replies.isNotEmpty) ...[
                            const SizedBox(width: 16),
                            TextButton.icon(
                              onPressed: () {
                                setState(() {
                                  _showReplies = !_showReplies;
                                });
                              },
                              icon: Icon(_showReplies ? Icons.expand_less : Icons.expand_more),
                              label: Text('${widget.comment.replies.length} replies'),
                            ),
                          ],
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
            if (_showReplies && widget.comment.replies.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(left: 40, top: 8),
                child: Column(
                  children: widget.comment.replies.map((reply) {
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 8),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          CircleAvatar(
                            radius: 12,
                            backgroundColor: Colors.grey.shade200,
                            child: Text(reply.author[0], style: const TextStyle(fontSize: 10)),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Row(
                                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                  children: [
                                    Text(
                                      reply.author,
                                      style: const TextStyle(fontWeight: FontWeight.bold),
                                    ),
                                    Text(
                                      dateFormat.format(reply.publishedAt),
                                      style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 4),
                                Text(reply.text),
                                const SizedBox(height: 4),
                                Row(
                                  children: [
                                    const Icon(Icons.thumb_up_outlined, size: 14),
                                    const SizedBox(width: 4),
                                    Text('${reply.likeCount}', style: const TextStyle(fontSize: 12)),
                                  ],
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    );
                  }).toList(),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
