"""
AgentMem Python SDK Setup
"""

from setuptools import setup, find_packages
import os

# Read README
def read_readme():
    with open("README.md", "r", encoding="utf-8") as fh:
        return fh.read()

# Read requirements
def read_requirements():
    with open("requirements.txt", "r", encoding="utf-8") as fh:
        return [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="agentmem",
    version="7.0.0",
    author="AgentMem Team",
    author_email="support@agentmem.dev",
    description="Official Python SDK for AgentMem - Enterprise-grade memory management for AI agents",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/agentmem/agentmem",
    project_urls={
        "Documentation": "https://docs.agentmem.dev",
        "Source": "https://github.com/agentmem/agentmem",
        "Tracker": "https://github.com/agentmem/agentmem/issues",
    },
    packages=find_packages(),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Database",
    ],
    python_requires=">=3.8",
    install_requires=read_requirements(),
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "isort>=5.12.0",
            "flake8>=6.0.0",
            "mypy>=1.0.0",
            "pre-commit>=3.0.0",
        ],
        "docs": [
            "sphinx>=6.0.0",
            "sphinx-rtd-theme>=1.2.0",
            "sphinx-autodoc-typehints>=1.22.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "agentmem=agentmem.cli:main",
        ],
    },
    include_package_data=True,
    package_data={
        "agentmem": ["py.typed"],
    },
    keywords=[
        "ai",
        "memory",
        "agent",
        "llm",
        "vector",
        "database",
        "embedding",
        "semantic",
        "search",
        "enterprise",
    ],
    zip_safe=False,
)
