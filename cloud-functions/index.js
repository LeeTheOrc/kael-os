const functions = require('firebase-functions');
const admin = require('firebase-admin');
const { GoogleGenerativeAI } = require('@google/generative-ai');

admin.initializeApp();

// Initialize Gemini with API key from environment
const genAI = new GoogleGenerativeAI(functions.config().gemini.key);

/**
 * Scheduled Cloud Function - runs daily to generate brainstorming ideas
 * Configure schedule: firebase functions:config:set scheduler.enabled=true
 * Deploy: firebase deploy --only functions:dailyBrainstorm
 */
exports.dailyBrainstorm = functions.pubsub
  .schedule('0 2 * * *') // 2 AM daily (adjust timezone as needed)
  .timeZone('America/New_York') // Change to your timezone
  .onRun(async (context) => {
    console.log('🧠 Starting daily brainstorm generation...');
    
    try {
      // Use cheapest Gemini model (gemini-1.5-flash)
      const model = genAI.getGenerativeModel({ model: 'gemini-1.5-flash' });
      
      // Brainstorming prompts - varied to get different perspectives
      const prompts = [
        {
          category: 'features',
          prompt: 'Generate 3 innovative feature ideas for Kael-OS, an AI-powered terminal assistant. Focus on developer productivity and creative workflows. Keep each idea concise (2-3 sentences).'
        },
        {
          category: 'ui',
          prompt: 'Suggest 3 UI/UX improvements for an AI terminal application. Focus on visual elegance, accessibility, and user delight. Keep each idea concise.'
        },
        {
          category: 'optimization',
          prompt: 'Propose 3 performance or workflow optimizations for an AI assistant that integrates local and cloud LLMs. Focus on speed, efficiency, and smart caching. Keep each idea concise.'
        },
        {
          category: 'integration',
          prompt: 'Suggest 3 creative integrations for Kael-OS with existing developer tools or services (GitHub, VS Code, Docker, etc.). Keep each idea concise.'
        }
      ];
      
      const db = admin.firestore();
      const batch = db.batch();
      const timestamp = admin.firestore.FieldValue.serverTimestamp();
      
      // Generate ideas for each category
      for (const { category, prompt } of prompts) {
        try {
          const result = await model.generateContent(prompt);
          const response = await result.response;
          const text = response.text();
          
          // Store in Firestore
          const docRef = db.collection('brainstorm_cache').doc();
          batch.set(docRef, {
            category,
            prompt,
            ideas: text,
            generated_at: timestamp,
            status: 'active',
            starred: false
          });
          
          console.log(`✅ Generated ${category} ideas`);
        } catch (error) {
          console.error(`❌ Failed to generate ${category} ideas:`, error);
        }
      }
      
      // Commit all to Firestore
      await batch.commit();
      
      // Cleanup old ideas (keep last 7 days)
      const sevenDaysAgo = new Date();
      sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);
      
      const oldDocs = await db.collection('brainstorm_cache')
        .where('generated_at', '<', sevenDaysAgo)
        .where('starred', '==', false)
        .get();
      
      const deleteBatch = db.batch();
      oldDocs.forEach(doc => deleteBatch.delete(doc.ref));
      await deleteBatch.commit();
      
      console.log(`🗑️ Cleaned up ${oldDocs.size} old ideas`);
      console.log('✨ Daily brainstorm complete!');
      
      return null;
    } catch (error) {
      console.error('💥 Brainstorm function error:', error);
      throw error;
    }
  });

/**
 * On-demand brainstorm - call from app when user wants fresh ideas
 * Usage: POST to function URL with { category: "features" | "ui" | etc }
 */
exports.onDemandBrainstorm = functions.https.onCall(async (data, context) => {
  // Require authentication
  if (!context.auth) {
    throw new functions.https.HttpsError(
      'unauthenticated',
      'User must be authenticated to generate ideas'
    );
  }
  
  const { category = 'features', customPrompt } = data;
  
  console.log(`🎯 On-demand brainstorm for ${context.auth.uid} - ${category}`);
  
  try {
    const model = genAI.getGenerativeModel({ model: 'gemini-1.5-flash' });
    
    // Default prompts by category
    const categoryPrompts = {
      features: 'Generate 3 innovative feature ideas for Kael-OS, an AI-powered terminal assistant. Focus on developer productivity.',
      ui: 'Suggest 3 UI/UX improvements for an AI terminal application. Focus on visual elegance and user delight.',
      optimization: 'Propose 3 performance optimizations for an AI assistant. Focus on speed and efficiency.',
      integration: 'Suggest 3 creative integrations with developer tools (GitHub, VS Code, Docker, etc.).',
      custom: customPrompt || 'Generate 3 creative ideas for improving a developer tool.'
    };
    
    const prompt = categoryPrompts[category] || categoryPrompts.features;
    const result = await model.generateContent(prompt);
    const response = await result.response;
    const text = response.text();
    
    // Store in Firestore
    const db = admin.firestore();
    const docRef = await db.collection('brainstorm_cache').add({
      category,
      prompt,
      ideas: text,
      generated_at: admin.firestore.FieldValue.serverTimestamp(),
      user_id: context.auth.uid,
      status: 'active',
      starred: false,
      on_demand: true
    });
    
    console.log(`✅ Generated on-demand ${category} ideas: ${docRef.id}`);
    
    return {
      success: true,
      id: docRef.id,
      category,
      ideas: text
    };
  } catch (error) {
    console.error('💥 On-demand brainstorm error:', error);
    throw new functions.https.HttpsError('internal', error.message);
  }
});

/**
 * Star/unstar an idea to keep it permanently
 */
exports.toggleStarIdea = functions.https.onCall(async (data, context) => {
  if (!context.auth) {
    throw new functions.https.HttpsError('unauthenticated', 'Must be authenticated');
  }
  
  const { ideaId, starred } = data;
  
  if (!ideaId) {
    throw new functions.https.HttpsError('invalid-argument', 'ideaId is required');
  }
  
  try {
    const db = admin.firestore();
    await db.collection('brainstorm_cache').doc(ideaId).update({
      starred: starred === true,
      starred_at: starred ? admin.firestore.FieldValue.serverTimestamp() : null,
      starred_by: starred ? context.auth.uid : null
    });
    
    console.log(`⭐ Idea ${ideaId} ${starred ? 'starred' : 'unstarred'} by ${context.auth.uid}`);
    
    return { success: true, starred };
  } catch (error) {
    console.error('💥 Toggle star error:', error);
    throw new functions.https.HttpsError('internal', error.message);
  }
});
