
package com.abode.wordandroid

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            WordAndroidApp()
        }
    }
}

@Composable
fun WordAndroidApp() {
    var selectedTab by remember { mutableStateOf("Home") }
    var documentText by remember { mutableStateOf("") }

    val tabs = listOf(
        "File", "Home", "Insert", "Layout",
        "References", "Review", "View", "Help"
    )

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFFE8EAF0))
    ) {
        // Title bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Color(0xFF185ABD))
                .padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                "W",
                color = Color.White,
                fontSize = 25.sp
            )
            Spacer(Modifier.width(12.dp))
            Text(
                "Document1",
                color = Color.White,
                fontSize = 18.sp
            )
            Spacer(Modifier.weight(1f))
            Text("Save", color = Color.White)
        }

        // Main tabs
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Color.White)
                .horizontalScroll(rememberScrollState())
        ) {
            tabs.forEach { tab ->
                Column(
                    modifier = Modifier
                        .clickable { selectedTab = tab }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Text(
                        tab,
                        color = if (selectedTab == tab)
                            Color(0xFF185ABD) else Color.DarkGray
                    )
                    if (selectedTab == tab) {
                        Spacer(Modifier.height(5.dp))
                        Box(
                            Modifier
                                .width(30.dp)
                                .height(3.dp)
                                .background(Color(0xFF185ABD))
                        )
                    }
                }
            }
        }

        // Ribbon
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Color(0xFFF8F9FB))
                .horizontalScroll(rememberScrollState())
                .padding(10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val tools = when (selectedTab) {
                "Home" -> listOf(
                    "Paste", "Cut", "Copy", "Font", "Size",
                    "Bold", "Italic", "Underline", "Align"
                )
                "Insert" -> listOf(
                    "Table", "Picture", "Shapes", "Link", "Page"
                )
                "Layout" -> listOf(
                    "Margins", "Orientation", "Size", "Columns"
                )
                "References" -> listOf(
                    "Contents", "Footnote", "Citation", "Index"
                )
                "Review" -> listOf(
                    "Spelling", "Comment", "Track Changes"
                )
                "View" -> listOf(
                    "Zoom", "Page Width", "Ruler"
                )
                else -> listOf("Open", "New", "Save", "Print")
            }

            tools.forEach { tool ->
                Surface(
                    modifier = Modifier.padding(end = 6.dp),
                    color = Color.White,
                    shape = RoundedCornerShape(6.dp),
                    shadowElevation = 1.dp
                ) {
                    Text(
                        tool,
                        modifier = Modifier.padding(
                            horizontal = 12.dp,
                            vertical = 10.dp
                        ),
                        fontSize = 12.sp,
                        color = Color.DarkGray
                    )
                }
            }
        }

        // Document workspace
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(12.dp),
            contentAlignment = Alignment.TopCenter
        ) {
            Surface(
                modifier = Modifier
                    .fillMaxHeight()
                    .fillMaxWidth(0.88f),
                color = Color.White,
                shadowElevation = 3.dp
            ) {
                BasicTextField(
                    value = documentText,
                    onValueChange = { documentText = it },
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(32.dp),
                    textStyle = androidx.compose.ui.text.TextStyle(
                        color = Color.Black,
                        fontSize = 16.sp,
                        lineHeight = 25.sp
                    ),
                    decorationBox = { innerTextField ->
                        Box {
                            if (documentText.isEmpty()) {
                                Text(
                                    "Start typing your document...",
                                    color = Color.LightGray,
                                    fontSize = 16.sp
                                )
                            }
                            innerTextField()
                        }
                    }
                )
            }
        }

        // Status bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Color(0xFFF8F9FB))
                .padding(horizontal = 14.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text("Page 1 of 1", fontSize = 11.sp)
            Text("${documentText.length} characters", fontSize = 11.sp)
            Text("100%", fontSize = 11.sp)
        }
    }
}
